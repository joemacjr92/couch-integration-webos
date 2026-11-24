# Persistent webOS WebSocket receiver investigation

## Scope

This branch investigates a persistent receive loop for the LG webOS SSAP and pointer WebSocket connections. It is intentionally independent of the mute-toggle PR. The goal is to make unsolicited TV notifications available to the integration continuously, so state does not depend on an unrelated command happening to read from the socket.

This document records the design constraints before changing transport code.

## Reference behavior

AioWebOSTV (used by Home Assistant and albinator) has:

- A long-lived main SSAP WebSocket and a separate input WebSocket.
- A dedicated receive task for each connection.
- Request/response correlation by message ID.
- Subscription notifications routed to registered callbacks.
- Subscription setup that registers its handler before sending the subscribe request.
- Connection teardown that cancels receiver/callback tasks, pending futures, and closes both sockets.

Relevant implementation: `aiowebostv/webos_client.py`, especially `connect_handler`, `_rx_msgs_main_ws`, `_process_text_message`, `request`, and `subscribe`.

## Couch constraints discovered

The integration is built against Couch SDK commit
`06377bfee341321fe13164607ecb98787c4c522b`.

The SDK's `DeviceClient` contract is synchronous. Its documentation says a client instance owns its transport, is not shared between threads, and is expected not to add its own request queue; the Couch broker serializes access. The package server also processes one framed request at a time and invokes `connect_with`, `execute`, `status`, `inputs`, or `apps` synchronously.

The package protocol currently has no unsolicited event frame from an integration to the host. A background receiver can therefore keep the integration's internal state current, but cannot independently push a state update into Couch. Couch will observe that state on its next status request.

This distinction must remain explicit: continuous receive is not, by itself, a host-side push notification feature.

## Superseded worker proposal

An earlier candidate proposed moving socket ownership to a background worker and handing synchronous calls to it through a rendezvous channel. The feasibility review below rejects that design for the pinned SDK: it introduces a second thread and an internal dispatch mechanism despite the SDK's ownership/no-queue contract. It is retained here only as investigation history, not as an implementation recommendation.

## Important implementation questions

- Does the pinned SDK's "no internal queuing" rule permit a dedicated transport owner with a zero-capacity handoff, or should this require a SDK contract/API change? Do not assume this away.
- What read strategy can continuously service incoming frames without a busy loop and without corrupting tungstenite's framing state on socket timeouts?
- How will the worker handle Ping/Pong, close frames, malformed JSON, unknown IDs, and subscription messages with no pending command?
- Which webOS subscriptions are supported reliably across target firmware? Start with audio status for mute/volume; add other state subscriptions only after protocol behavior is verified.
- What is the exact resubscription policy after a dropped socket? A reconnect must use the stored client key and pinned certificate and must not initiate pairing.
- How will worker shutdown interrupt a blocked read and guarantee the thread is joined?
- How will transport errors map to SDK errors so the package server retires the unusable client as intended?

## Proposed implementation sequence

1. Add transport-worker primitives and unit tests for message routing using a fake WebSocket peer.
2. Move the main socket behind the worker without changing command semantics.
3. Add continuous receive and request-ID correlation.
4. Add audio-status subscription and notification-derived mute state.
5. Move pointer socket ownership into the same lifecycle and test pointer traffic.
6. Add disconnect, timeout, resubscribe, and shutdown tests.
7. Run formatting, all-target tests, release build, and ARMv7 CI.
8. Only then use the physical TV to validate persistent operation, mute toggles, navigation, and recovery after TV/network disconnects.

## Non-goals

- No changes to PR #4 or its native MUTE-key experiment.
- No host/Couch protocol changes in this branch unless investigation proves they are required and the change is separately agreed.
- No automatic retry of non-idempotent TV commands.
- No claim of immediate UI updates until Couch exposes an event path from the package to the host.

## Feasibility decision against the pinned SDK

The candidate worker design above is **not compatible with the current SDK contract as written**. This is no longer an open question based on interpretation: the pinned `DeviceClient` documentation says that the client instance owns its transport, is never shared between threads, and assumes that the client itself does no internal queuing. The zero-capacity handoff still introduces a second thread and an internal command-dispatch mechanism, so calling it a rendezvous channel does not make it compliant.

The pinned package server reinforces this boundary. It owns one mutable `Option<C>` on its serving thread and calls `connect_with`, `execute`, `status`, `inputs`, and `apps` synchronously while processing one framed host request at a time. On configure it drops the client; on later device requests it lazily creates a new one. There is no callback/event sink exposed to a client, and the protocol has no unsolicited package-to-host state event.

### Consequence

Do not implement the worker in this integration against the current SDK. It would violate an explicit SDK assumption and still would not provide immediate Couch UI updates because the package has no event path to the host.

### Supported paths

1. **Stay within the current SDK:** keep socket ownership and all I/O on the calling thread. Process unsolicited SSAP notifications whenever a synchronous request is reading frames, retain subscription-derived state for the next status call, and preserve request-ID correlation. This can improve correctness during active traffic, but it cannot receive while the package is idle.
2. **Change the SDK and package protocol upstream:** introduce an explicit asynchronous/event-capable client contract and host event channel. The SDK would need to define lifecycle, cancellation, reconnection, event backpressure, and whether a separate transport task is permitted. The package server and host must both support delivering state events. Only after that contract exists should this integration add a persistent receiver.

The first path is the only integration-only change that fits the pinned SDK. The second is the proper route if the actual requirement is continuously updated state while idle. Do not disguise path 1 as a persistent background receiver, and do not implement path 2 locally in this integration.

## Next implementation decision

Proceed with path 1 as a bounded, test-first improvement: add explicit handling for known SSAP subscription notifications encountered during ordinary response reads, without adding threads, channels, or asynchronous claims. Add fake-peer tests for notification-before-response, unrelated response IDs, malformed messages, and timeout behavior. Keep subscription registration and cached state scoped to the lifetime of the existing synchronous connection. Revisit a true always-on receiver only after Couch SDK and package protocol provide the required contract.


## Synchronous subscription investigation

The reference client confirms two useful SSAP subscriptions:

- `ssap://audio/getStatus` — mute state is exposed as `payload.mute` in the reference callback.
- `ssap://audio/getVolume` — volume is exposed as `payload.volumeStatus.volume` on newer firmware, with a compatibility fallback to `payload.volume` on older firmware. The complete volume-status object is retained by the reference client.

Both are sent as SSAP messages with `type: "subscribe"`, not ordinary `type: "request"`. The reference client allocates a stable message ID and registers the notification handler before sending the subscribe message. The first message for that ID establishes the subscription result/state; later messages carrying that same ID are delivered to the subscription handler. This means a synchronous implementation must reserve and recognize the subscription ID for the entire connection lifetime, rather than treating the first response as the end of that ID's lifecycle.

### Integration-specific implications

The current `WebOsTv::request` sends `type: "request"`, waits for one matching ID, and currently discards every message with a different ID. A subscription implementation cannot safely be added by simply changing the message type: it needs a small message dispatcher in the existing synchronous read path.

Recommended design, still on the calling thread:

1. Add a typed subscription registry keyed by SSAP message ID (and optionally URI for diagnostics). Initially support only audio status and volume status.
2. Add a `subscribe` helper that allocates a unique ID, sends `type: "subscribe"`, waits for that ID's successful initial response, extracts the initial state, and only then retains the ID in the registry. Do not register after sending if there is any possibility of reading frames between send and registration; reserve the ID first, then mark it active after success.
3. In the normal request receive loop, route messages whose IDs belong to active subscriptions into a pure state-update function, then continue waiting for the normal request's ID. Never let a subscription notification satisfy an unrelated request.
4. Preserve the existing per-request deadline. A notification must not reset the deadline or extend a command indefinitely.
5. Keep current explicit `getVolume` reads initially. Subscription state is opportunistic because no code reads the socket while the package is idle. Use it to validate notification parsing and to keep the most recently observed mute/volume snapshot coherent during active traffic; do not claim it is always fresh.
6. If a subscription fails or is rejected, keep ordinary request/status behavior working. Subscription support is an enhancement, not a reason to make pairing or the entire connection unusable.

### Important nuance: subscription IDs and payloads

The reference implementation uses one ID per subscription and routes messages by that ID. The notification payload schema is endpoint-specific. For audio status, the mute field is `mute`; for volume, normalize `volumeStatus` when present and otherwise use the top-level payload. Do not assume the volume subscription contains mute state or that `getVolume` always has the same shape across firmware.

The integration currently has a `mute_state` helper that accepts `muted` and `mute`. Keep that compatibility behavior, but add a separate normalizer for the volume-status envelope rather than conflating the two endpoint schemas.

### Test matrix before implementation

- Subscribe sends `type: "subscribe"` with a unique ID and accepts a successful initial subscription response.
- Initial audio-status payload populates mute state from `mute` (including explicit `false`).
- Volume payload normalizes both nested `volumeStatus` and legacy top-level fields.
- A subscription notification arriving before an ordinary request's response updates cached state and does not complete the ordinary request.
- An unrelated/unknown ID is ignored without corrupting the pending request.
- A subscription notification with malformed or invalid fields does not erase a previously valid cached value.
- A rejected/unsupported subscription leaves ordinary `getVolume` and status requests functional.
- A request timeout remains bounded even if subscription notifications continue arriving.
- Subscription registry and cached state are discarded with the WebSocket/client; no stale state crosses reconnects.

### Synchronous subscription implementation status

The first integration-only implementation is now present in `src/lib.rs`.

- On successful authenticated connection, the client makes best-effort subscriptions to `ssap://audio/getStatus` and `ssap://audio/getVolume`.
- Each subscription uses its own monotonically allocated SSAP message ID and sends `type: "subscribe"`.
- The initial response must be a successful `response` with one of the reference client's success indicators: `returnValue: true`, `subscribed: true`, or `subscription: true`. An explicit rejection or malformed acknowledgement leaves that subscription inactive.
- Successful subscription IDs are retained for the lifetime of the `WebOsTv` instance. During an ordinary request read, frames for those IDs are dispatched to the corresponding audio-state parser; the pending command continues waiting for its own ID.
- Volume notifications accept both the nested `volumeStatus` envelope and the legacy top-level shape. Audio status uses the `mute` field. Only valid values update the cache, so malformed updates do not erase prior state.
- The ordinary `getVolume` request remains in place. The cached values are fallback state only when a response omits a field; they are not represented as continuously fresh.
- Rejected, malformed, or timed-out subscriptions are optional and do not prevent ordinary operation. A transport failure during setup is fatal, because the client cannot safely be returned with a broken socket.

Unit tests now cover nested and legacy volume payloads, explicit false mute values, malformed-value preservation, subscription-ID routing, unknown IDs, and acknowledgement success/rejection parsing.

### Remaining verification

The current tests exercise the pure parsing and dispatch logic. They do **not yet** exercise the actual WebSocket exchange with a fake peer, prove notification interleaving against a live socket, or prove timeout behavior under a continuous stream of notifications. Those remain required before treating the transport behavior as fully verified. The branch admission workflow is configured for pull requests and pushes to `main`, not arbitrary feature-branch pushes, so this commit has not automatically run that workflow. No local Rust toolchain/test execution was available through the GitHub editing path; formatting, `cargo test --locked --all-targets`, and the ARMv7 release build remain unverified.

## Scope boundary

This path is **not** a persistent receiver. It only processes notifications when a synchronous operation is already reading the WebSocket. It is a useful, testable improvement compatible with the current SDK, but it does not solve idle-time state freshness or immediate Couch UI updates. Those require the upstream asynchronous SDK and package-event protocol work described above.
