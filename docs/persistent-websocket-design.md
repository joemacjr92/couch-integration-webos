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

## Candidate design

Use a single transport-owner worker for the lifetime of a connected `WebOsTv`:

1. The worker owns the main SSAP socket and pointer socket. No other thread reads or writes either WebSocket.
2. The synchronous `DeviceClient` methods submit one command to the worker and wait for its result.
3. Use a zero-capacity rendezvous channel (or equivalent one-request handoff), not an unbounded command backlog. The Couch broker already serializes client calls.
4. The worker continuously reads main-socket frames between command handoffs. It dispatches response frames by request ID and processes subscription notifications independently.
5. A shared, narrowly scoped state snapshot holds notification-derived values such as mute, volume, power, foreground app, and sound output. Status can combine that snapshot with explicit reads where necessary.
6. Pointer input is sent by the same owner so the pointer socket is never concurrently accessed. Its receive path handles control frames and connection closure.
7. Dropping/replacing the client sends shutdown, closes sockets, and joins the worker with a bounded deadline.

The worker must not silently retry commands. A timeout or disconnect is an uncertain outcome and must be surfaced to the caller; reconnect/resubscribe belongs to a separately specified lifecycle policy.

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
