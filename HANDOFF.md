# CouchOS webOS Integration — Chat Handoff

Last updated: 2026-11-25

This document is the continuity reference for future ChatGPT sessions working on the CouchOS LG webOS integration. Read it before proposing changes, running commands, or discussing pull requests. It exists to keep the repository, branch, and validation state unambiguous.

## Repository identity

- User's fork: https://github.com/joemacjr92/couch-integration-webos
- Upstream: https://github.com/Couch-OS/couch-integration-webos
- Primary development branch in the fork: `main`
- Handoff-only branch: `docs/chat-handoff` (this branch)
- Upstream contribution branch: `upstream/webos-functional-improvements`
- Upstream contribution PR: [#6 — Add HDMI-CEC input discovery and improve LG webOS integration](https://github.com/Couch-OS/couch-integration-webos/pull/6)

Do not confuse this project with the separate Fire TV or Monoprice HTP-1 integrations. They are useful examples for conventions and documentation style, but changes to those repositories are outside this project's scope unless explicitly requested.

## Branch and PR map

| Ref | Purpose | State / guidance |
|---|---|---|
| `main` in the fork | Current integrated development baseline | Source tree used for the upstream contribution |
| `upstream/webos-functional-improvements` | Single-commit upstream contribution branch | Created from upstream `main`; contains the exact tracked tree from the fork's tested `main` |
| Upstream PR #6 | Review of the functional improvements | Open as of this handoff; do not create a duplicate PR |
| `docs/chat-handoff` | Chat continuity documentation | Keep handoff notes here; do not use it as the upstream feature PR branch |
| Fork PR #4 | Native LG MUTE key for mute toggle | Separate work; CI and physical TV validation were still pending when last checked |
| Fork PR #5 | GitHub Actions build/signing-key cutover preparation | Separate workflow/repository-metadata work; do not conflate with upstream PR #6 |

### Upstream contribution branch details

- Branch: `upstream/webos-functional-improvements`
- Commit: `d2e4d6203e996b38efcaa7c9aeb605a2a4734637`
- Commit message: `feat(webos): add HDMI-CEC discovery and improve TV controls`
- Parent: upstream `Couch-OS/couch-integration-webos:main` at `5560f66538a9d41e84ba33aa125a3aa55b4cad37`
- Tree: exact tree from the fork's `main` at `fbb83c1ecafc5a14da038e23d7f27b3aeb6deb95`
- The branch was deliberately condensed to one commit for upstream review. The tree comparison showed the same 12 changed files and change counts as the fork's `main` compared with upstream, with no commits behind upstream at creation time.
- This branch includes the features, tests, documentation, release/feed infrastructure, and workflows already present on the fork's `main` at that point. It does **not** include separate unmerged work that was not on `main`, including the later native pointer-input MUTE toggle work.
- PR #6: https://github.com/Couch-OS/couch-integration-webos/pull/6
- Compare view: https://github.com/Couch-OS/couch-integration-webos/compare/main...joemacjr92:upstream/webos-functional-improvements

Do not rewrite or force-push the contribution branch without first inspecting the current PR, its review comments, and the latest remote refs.

## Functional scope included in upstream PR #6

The submitted tree includes the following work from the fork's tested `main`:

- HDMI-CEC device discovery and exposure as selectable inputs.
- Preservation of the parent TV input context while showing discovered CEC devices.
- CEC identity/topology validation to avoid selecting stale devices.
- More reliable input switching, including settling behavior during CEC transitions.
- Audio mute-state handling and explicit mute-on / mute-off operations as implemented on `main`.
- Volume-level reporting through LG audio status and volume subscriptions, with fallback behavior when optional subscriptions are unavailable.
- WebSocket subscription routing and connection-failure handling improvements.
- Expanded unit and admission tests for CEC discovery, input selection, audio state, subscriptions, and failure scenarios.
- Documentation of audio subscription behavior and limitations.

Important distinction: the separate native LG pointer-input `MUTE` key implementation is not part of PR #6. It is tracked separately in fork PR #4. Do not describe PR #6 as containing that native-key toggle.

## Validation recorded for the source tree

The fork's `main` tree used to construct the squashed contribution branch had a successful CI run:

- CI run: https://github.com/joemacjr92/couch-integration-webos/actions/runs/45552001053
- `cargo fmt --all -- --check`
- `cargo test --locked --all-targets`
- Unit tests: 17 passed.
- Admission tests: 7 passed.
- Static ARMv7 package build passed.
- Signed feed-history and repository public-key validation passed.

The squashed contribution branch was created with the exact tree from that tested `main`, but a separate CI run on the squashed branch had not been confirmed at the time of this handoff. Phrase this accurately: validation passed on the source development branch; do not claim a separate run on the squashed branch unless a newer run verifies it.

## Audio and mute behavior — important implementation boundaries

- Audio subscriptions use `ssap://audio/getStatus` and `ssap://audio/getVolume`.
- The cached audio state tracks optional mute and volume values.
- Subscription payload processing occurs during ordinary TV communication. It does not provide independent background updates while the Couch client is idle.
- Unsupported optional audio subscriptions should not prevent normal TV control; actual transport failures remain meaningful failures.
- On the fork's `main` at the time of the contribution, mute toggle used the cached mute state and `ssap://audio/setMute`.
- Fork PR #4 changes Toggle to use the TV's native pointer-input `MUTE` key, leaving explicit Mute On/Off on `ssap://audio/setMute`. Its PR description states CI and physical LG TV validation were still required.
- A separate parser improvement discussed in prior work handles both `muted` and `mute` response fields. Verify whether it has since been merged before assuming it is in `main` or PR #6.

## Upstream contribution conventions

At the time of preparation:

- No upstream `CONTRIBUTING.md` or PR template was found.
- Existing upstream PR descriptions generally use `## Summary` and `## Validation`, with compatibility/dependency notes when relevant.
- The upstream admission workflow checks the immutable SDK pin/package metadata, formatting, locked all-target tests, and the static ARMv7 package build.
- Keep PR descriptions factual and distinguish tests actually run from tests inferred from an identical tree.

## Working rules for future sessions

1. Start by checking the live GitHub state. Branches, PRs, CI runs, and merge status may have changed since this document was written.
2. Read this handoff before making assumptions about which repository or branch is under discussion.
3. Preserve all existing features, tests, workflows, package/feed infrastructure, and documentation unless the user explicitly authorizes a scoped removal.
4. Do not silently move changes between `main`, the upstream contribution branch, and separate feature PRs.
5. Before editing, identify the exact target branch and inspect its current contents and status.
6. Do not claim physical-device validation unless the user has supplied evidence that it was performed.
7. Do not claim CI passed on a branch unless that branch has a corresponding successful run, or clearly explain that the tested tree is identical.
8. Show actual code changes when requested; do not substitute test-only changes when the user asks for implementation changes.
9. Keep Fire TV and HTP-1 repositories untouched when using them only as examples.
10. Ask before creating, updating, closing, merging, or retargeting PRs. The user generally wants a ready-to-use link or draft, but performs the final upstream PR action themselves unless they explicitly ask otherwise.

## First checks in a new chat

Before continuing work, verify:

- Current fork `main` SHA and working-tree/remote status if local access is available.
- Whether upstream PR #6 is still open, merged, or has review feedback.
- Whether `upstream/webos-functional-improvements` still points to the expected commit.
- Current CI status for PR #6.
- Whether fork PRs #4 and #5 have changed state.
- Whether the mute parser/native MUTE work has since landed in `main`.

Useful links:

- Fork: https://github.com/joemacjr92/couch-integration-webos
- Upstream: https://github.com/Couch-OS/couch-integration-webos
- Upstream PR #6: https://github.com/Couch-OS/couch-integration-webos/pull/6
- Fork PR #4: https://github.com/joemacjr92/couch-integration-webos/pull/4
- Fork PR #5: https://github.com/joemacjr92/couch-integration-webos/pull/5
- Source-tree CI run: https://github.com/joemacjr92/couch-integration-webos/actions/runs/45552001053


---

# Cross-Integration Engineering Handoff

This section captures the shared engineering assumptions, decisions, and lessons from building all three CouchOS integrations. It is intentionally broader than the webOS feature handoff. Future sessions should use these as starting constraints, while still checking the live repositories before treating any implementation detail as current.

## The three projects

| Integration | Repository | Device / protocol focus | Proven scope |
|---|---|---|---|
| Fire TV | https://github.com/joemacjr92/couch-integration-firetv | Amazon Fire TV local REST API, PIN pairing, HTTPS | Navigation, media controls, supported wake behavior |
| Monoprice HTP-1 | https://github.com/joemacjr92/couch-integration-htp1 | HTP-1 local WebSocket controller API | Power, volume, mute, input, status, five custom upmixer commands |
| LG webOS | https://github.com/joemacjr92/couch-integration-webos | LG webOS SSAP/WebSocket and pointer-input behavior | TV controls, HDMI-CEC discovery/selection, audio state/subscriptions |

These projects are separate integrations with different device protocols and capabilities. Reuse engineering patterns where appropriate; do not assume commands, connection models, authentication, event delivery, or capability sets are interchangeable.

## Shared CouchOS assumptions and decisions

### 1. Build for the actual CouchOS runtime and package contract

- Treat the CouchOS SDK, package metadata, protocol version, target architecture, and packaging workflow as part of the product contract—not incidental build details.
- Keep SDK references pinned and reproducible. Do not casually update SDK revisions or package metadata while making an unrelated feature change.
- ARMv7 is the target architecture used by these integrations. A successful host build or unit-test run does not establish that the deployable ARMv7 package builds.
- Validate with the repository's locked Rust dependency graph and the actual package/admission workflow. Avoid unintentional dependency drift.
- Preserve the established package identity and metadata. Package IDs matter to repository installation and migration behavior.
- Do not add development-machine setup details to user-facing READMEs. Document what an end user needs to install, configure, use, and troubleshoot the integration.

### 2. Separate package distribution trust from device authentication

There are two different security concerns:

- **Package repository trust:** CouchOS verifies repository metadata/packages using the repository's public signing key and fingerprint.
- **Device authentication:** each device protocol has its own credentials or pairing model. For example, Fire TV PIN pairing produces device credentials managed through CouchOS.

Never conflate these. Never publish or commit a private package-signing key or private device credential. Publish only the repository public key. Users should independently compare the fingerprint CouchOS displays with the fingerprint published in the integration's documentation before trusting a repository.

Fingerprint formats can be implementation-specific. The HTP-1 README explicitly documents the normalized PEM SHA-256 fingerprint shown by CouchOS and warns that it is not the DER public-key fingerprint. Do not relabel or recalculate a fingerprint without confirming the exact CouchOS calculation.

### 3. Trusted repository installation is a package lifecycle, not just adding a URL

The Fire TV and HTP-1 migration tests established an important operational behavior:

- A locally sideloaded package and a repository-managed package with the same integration/package ID are not automatically converted just because the repository is added.
- To make the repository-managed package available for selection/installation, remove the existing locally sideloaded package first.
- Removing the integration package preserved its saved CouchOS connection settings in the tested Fire TV and HTP-1 migrations.
- The Fire TV migration was deliberately scoped: remove only the Fire TV package; do not remove the webOS integration or its repository.
- The HTP-1 README now documents removing the sideloaded package before installing the repository-managed package with the same ID.

Practical rule: identify the exact package ID and installation source before recommending removal. Remove only the intended sideloaded package. Do not tell users to delete connection records, reset CouchOS, remove unrelated integrations, or remove a trusted repository unless there is specific evidence that is necessary.

Test this behavior per integration when package IDs, migration code, or CouchOS package management changes. The fact that saved connections survived the Fire TV and HTP-1 migrations is a verified observation for those tests, not a blanket guarantee for every future package migration.

### 4. A successful transport handshake is not proof of a working integration

Validate the full application protocol and user-visible behavior:

- An HTTP WebSocket upgrade response of `101` proves the WebSocket handshake, not that the application protocol, authentication, subscriptions, command handling, or state refresh works.
- A settings form saving without a visible error does not prove that the device connection works or that state is being retrieved.
- Verify actual command/response behavior, parsing, status refresh, error paths, and the behavior exposed to the CouchOS remote.
- Device-specific ports and paths must be configurable or documented according to the actual protocol. CouchOS has been observed using nonstandard ports for other integrations; do not assume every device uses port 80 or 443.
- Keep device communication on the local network where the device API is designed for local control. Document TLS behavior accurately, including any device-specific certificate limitations.

### 5. Distinguish command semantics and state semantics

Do not assume a command named Toggle is equivalent to explicit On/Off, or that cached state is always fresh enough to drive a toggle.

- Prefer the device's native operation when it is the most reliable and semantically correct mechanism.
- Explicit set-state operations and toggle operations may need different implementations.
- If a command depends on cached state, prove how and when that state is refreshed, whether updates can arrive independently, and what happens when the cache is stale or absent.
- Do not imitate another integration's asynchronous event model in a synchronous client without proving that the receive loop, message routing, and timing model are equivalent.
- Parse device responses defensively. Preserve valid cached values when optional or malformed fields are absent; do not turn missing optional data into fabricated state.
- Treat optional capability/subscription failures differently from core transport failures. Unsupported optional features should not unnecessarily disable basic control, but a genuine connection failure must not be hidden as an unsupported feature.

The webOS mute-toggle investigation is a concrete example: the cached-subscription approach depended on receive-loop behavior that did not map cleanly to Couch's synchronous client. The separate native LG pointer-input `MUTE` approach was chosen for Toggle, while explicit Mute On/Off remained SSAP `setMute` operations. Keep that distinction visible when discussing webOS branches and PRs.

### 6. Concurrency and ordering are correctness concerns

The HTP-1 work established that a persistent WebSocket is not simply a stateless request pipe:

- HTP-1 uses a persistent WebSocket with messages such as `getmso`, `mso`, `msoupdate`, and `changemso`.
- Commands are JSON patches, and write ordering matters.
- The agreed Phase 1 design is to serialize WebSocket operations that modify HTP-1 state while allowing safe independent read-only requests to remain concurrent.
- Preserve command payloads and sequencing while adding serialization. Test ordering and concurrent callers explicitly.
- Do not add a global lock indiscriminately if it blocks safe independent reads or creates avoidable latency.

This is an HTP-1-specific implementation decision that may inform other persistent WebSocket clients, but each protocol must be examined independently before applying the same locking model.

## Testing and evidence standards learned across the projects

### Test layers are complementary

Use the right evidence for each claim:

1. **Unit tests** — parsing, state transitions, command construction, edge cases, topology/identity validation, and error handling.
2. **Admission/package tests** — CouchOS-facing integration behavior, metadata, package process, simulated device interactions, and packaging assumptions.
3. **Cross-compilation/package build** — the actual target architecture and deployable artifact.
4. **CI** — reproducible execution of the repository's required checks.
5. **Physical-device testing** — actual pairing, command behavior, state reporting, timing, and compatibility on the target hardware.
6. **Remote/CouchOS UI verification** — package discovery, installation, connection configuration, and what the user can actually select or operate.

Passing one layer does not imply that the others passed. In particular, unit tests and simulated admission tests do not replace hardware validation; a successful package build does not prove the device protocol works.

### Use real logs to investigate real behavior

- Prefer captured CouchOS/device logs and actual command traces over speculation about latency, retries, polling, or protocol behavior.
- Trace the complete transaction path before changing timing or adding complexity.
- A delay should have a documented reason tied to a device transition or observed behavior. Do not increase a delay merely because a larger value seems safer.
- When evaluating a proposed timing change, compare captured behavior and tests, and ask whether it materially improves the user-visible outcome.
- Record what the evidence proves and what it does not prove. Avoid turning one successful trace into a universal compatibility claim.

### Preserve tests and existing behavior

- Before changing a tested integration, establish the baseline and identify the exact regression risk.
- Keep existing tests and add focused coverage for the new behavior. Do not replace implementation changes with tests when the requested work is an implementation fix.
- When the user says to preserve all features and tests, compare the complete target tree, not just the headline feature.
- For branch preparation, verify ancestry, changed files, commit counts, and tree equivalence where relevant. A squash should simplify history without dropping tracked files or behavior.
- Report the exact branch on which tests ran. If a branch has an identical tree to a tested branch but has not itself run CI, say that plainly.

## Project-specific verified lessons

### Fire TV

- Repository: https://github.com/joemacjr92/couch-integration-firetv
- The integration is a Rust port/adaptation of Mase (Meir Miyara)'s original Unfolded Circle Fire TV integration. Preserve attribution to Mase and to dangerouslaser, creator of CouchOS, its documentation, SDK, and integration ecosystem. Also retain relevant Fire TV REST API research attribution to SLC-Josh where appropriate.
- Pairing uses the Fire TV's four-digit PIN flow. Credentials are managed by CouchOS private credential storage, not ordinary integration settings.
- Communication uses the Fire TV local HTTPS API. The device API uses a self-signed TLS certificate; the integration accommodates this for local communication. This is a device-specific exception, not a pattern to copy blindly to other integrations.
- Verified controls include navigation (Up, Down, Left, Right, Select, Back, Home, Menu), media controls (Play/Pause, Fast Forward, Rewind), and supported power-on through DIAL.
- Do not claim unsupported capabilities: volume/mute, power-off, status reporting, input enumeration, app listing/launching, device discovery, and key-hold/repeat behavior were listed as not currently supported in the README.
- Initial design targeted Fire TV Cube; model and Fire OS API support can vary. Do not generalize compatibility to every Fire TV device.
- Repository ID: `firetv`; preview repository URL: https://joemacjr92.github.io/couch-integration-firetv/preview
- Published public-key fingerprint recorded in the README: `5ff5846b5d3cc1e404bd5a2ac7640414b91baecf20c2b03d0ada812c15b22a14`
- Tested migration: locally sideloaded `firetv 0.1.0` was removed; repository-managed `firetv 0.1.3` was then installed. Saved CouchOS connection settings remained, and webOS was left untouched.
- User-facing README should focus on installation, trust verification, pairing, supported features, compatibility, security, and credits—not local build-machine instructions.

### Monoprice HTP-1

- Repository: https://github.com/joemacjr92/couch-integration-htp1
- The integration is a Rust port/adaptation of Mase's original Unfolded Circle HTP-1 integration. Preserve attribution to Mase and dangerouslaser/CouchOS.
- HTP-1 communicates over its local WebSocket controller API. The controller path observed is `/ws/controller`; default port is `80`, with TLS optional and disabled by default in the documented configuration.
- A `101` HTTP upgrade alone is not a functional test. Confirm the controller protocol, state refresh, and command/response path.
- CouchOS connection settings are stored under `/opt/couch/connections/<integration-id>/`, not in a generic `config.json`. A tested HTP-1 connection record was at `/opt/couch/connections/monoprice-htp-1/plugin-connection.json`, with host `htp1-remote`, port `18081`, TLS disabled. Treat that as a test-environment observation, not a universal user configuration.
- Verified repository-managed version `0.1.1` on the CouchOS remote. Hardware validation confirmed power, volume, mute, input/status, and all five custom upmixer commands: Auro, Dolby Surround, DTS Neural:X, Native, and Stereo.
- Volume changes are in 1 dB steps. The protocol's safety behavior limits large jumps to 5 dB.
- The five upmixers are exposed as custom commands, not as a dynamic upmixer selector.
- Do not claim unsupported capabilities: Dirac Live calibration selection, Night mode, PEQ, seat-shaker controls, or a dynamic upmixer selector.
- Protocol requirement documented: CouchOS protocol 3 (minimum 3); package architecture ARMv7.
- Repository ID: `monoprice-htp1`; preview URL: https://joemacjr92.github.io/couch-integration-htp1/preview
- Preview feed path observed: `preview/armv7/feed.json`
- Published fingerprint in the README: `ad5ecdedddfbf48e8597053c2df0679c858af9ff4cdff879df50b9e33eeac79d` (normalized PEM fingerprint shown by CouchOS, not DER).
- Tested migration requires removing a sideloaded package with the same ID before installing the repository-managed package. The saved connection record was observed to persist.
- Agreed HTP-1 concurrency direction: serialize state-changing WebSocket writes, retain concurrency for safe independent read-only operations, and add ordering/concurrency tests. Verify the current branch before assuming this planned/accepted work has landed.
- A separate agreed compatibility direction was fail-open filtering for newer-firmware add-on status: retain inputs when add-on status is unknown while preserving older-firmware behavior. Check implementation status before treating it as complete.
- The user explicitly chose to establish SDK feasibility and core protocol behavior before broadening scope; media browsing was excluded from the initial scope.
- The README should match the Fire TV README's user-facing structure while accurately reflecting HTP-1 capabilities. Do not copy Fire TV's feature claims or device behavior.

### LG webOS

- Repository: https://github.com/joemacjr92/couch-integration-webos
- The Couch client is synchronous in important command/receive paths; do not assume Home Assistant's continuously running asynchronous WebSocket receiver exists in Couch.
- HDMI-CEC discovery and selection require preserving the parent TV input context and validating device identity/topology so stale CEC devices are not selected.
- Input switching includes a settle period for CEC transitions. Keep the delay tied to the measured transition behavior; do not casually remove or inflate it.
- Audio subscriptions are processed during ordinary TV communication, not by an independent background receiver while the client is idle.
- Optional unsupported audio subscriptions should not break basic TV control; distinguish that from transport failures.
- Volume and mute state reporting are separate from command semantics. The native LG pointer-input `MUTE` key was chosen for Toggle in separate fork PR #4; explicit Mute On/Off remain SSAP `ssap://audio/setMute` operations.
- Check current PR/branch state before assuming the native MUTE change or the response-field parser supporting both `muted` and `mute` has landed in `main`.

## Documentation and attribution standards

The Fire TV and HTP-1 READMEs established the preferred end-user documentation style:

1. Short CouchOS-focused introduction.
2. Clear trusted-repository installation steps, with repository ID, URL, public key, and fingerprint verification.
3. Pairing or connection configuration.
4. Supported features, with exact capabilities.
5. Compatibility and protocol/architecture requirements where known.
6. Explicitly listed unsupported features.
7. Security behavior, including local-network assumptions and credential handling.
8. Credits and acknowledgements for upstream/original work.

Additional rules:

- Keep documentation concise and user-facing. Do not include local development host paths, local build-machine setup, or irrelevant maintainer workflow details in end-user READMEs.
- Be precise about what has been tested on actual hardware versus simulated or CI-tested.
- Clearly state when compatibility is limited to a device family/model or protocol version.
- Preserve original authorship and attribution when porting/adapting another developer's integration. A Rust rewrite does not erase the original design or research contribution.
- Use the Fire TV and HTP-1 READMEs as structural examples for webOS, but never edit those READMEs merely to make them match webOS.

## Repository distribution and release lessons

- A preview feed is a real user-facing package distribution channel. Keep repository ID, base URL, architecture/feed paths, package metadata, versioning, public key, and generated index/feed files consistent.
- The public signing key belongs in the repository so users can verify the feed. The private signing key must remain secret (for example, in GitHub Actions Secrets) and must never be committed or exposed in logs.
- Validate the generated signed index/feed and package artifacts as part of release/admission checks. A successful Rust test run does not prove the signed feed is valid.
- Treat signing-key changes as a deliberate migration. Confirm the public/private key pair, fingerprint, feed-history behavior, and required Actions secrets before publishing. Do not reset feed history or publish a release unless explicitly authorized.
- Keep release workflow changes separate from feature work when possible, and document the required maintainer actions without exposing secrets.
- Never claim that a package was published, a feed updated, or a remote upgraded unless the relevant operation and resulting artifact/install have been verified.

## Future-session operating checklist across all three projects

Before acting:

1. Identify which of the three repositories the user means. Never infer from a generic phrase like "the integration" if context is ambiguous.
2. Read that repository's handoff and current README, then check live branches, PRs, workflows, and latest commits.
3. Identify the exact target branch and whether it is development, release, handoff, or upstream-submission work.
4. Check current package ID, repository ID, feed URL, SDK pin, protocol requirement, architecture, and signing-key fingerprint before changing packaging or install instructions.
5. Separate verified facts, observed device behavior, agreed design decisions, hypotheses, and future work. Label each accurately.
6. Preserve known working behavior and existing tests. Make the smallest justified change and add regression coverage.
7. Run the relevant test layers and report precisely which ones passed.
8. For device-dependent behavior, ask for or use real logs and physical-device results; never fabricate validation.
9. Keep the end-user README accurate and concise, including unsupported features and compatibility limits.
10. Preserve upstream attribution and do not alter sibling integration repositories unless explicitly asked.
11. Do not merge, publish, rotate signing keys, reset feed history, or perform destructive device/package operations without explicit authorization.
12. End each substantial task by updating the appropriate handoff with the decision, evidence, affected commit/PR, validation state, and unresolved follow-up—on the dedicated documentation branch, not accidentally in a feature/upstream PR.



---

# Current Status Addendum — 2026-11-25

This addendum records the pointer WebSocket investigation and the current repository/PR state. Where earlier sections below describe older branch or PR states, this addendum takes precedence. Re-check GitHub before relying on any status in a later session.

## Current repository and PR state

- User fork: https://github.com/joemacjr92/couch-integration-webos
- Upstream: https://github.com/Couch-OS/couch-integration-webos
- Fork `main` currently points to `db5c7bca105782e11d414f833e64f45027a9d601`.
- Upstream `main` currently points to `5560f66538a9d41e84ba33aa125a3aa55b4cad37`.
- Upstream PR #6 remains the single upstream contribution covering the HDMI-CEC and functional/audio improvements described above: https://github.com/Couch-OS/couch-integration-webos/pull/6
- Fork PR #11, “Continuously service the LG pointer WebSocket,” was squash-merged into the user's fork `main`: https://github.com/joemacjr92/couch-integration-webos/pull/11
- PR #11 merge commit is the current fork `main` SHA above. This was a fork merge only; it did not change upstream.
- Fork PR #10 (“Add pointer WebSocket recovery diagnostics”) and PR #8 (“Harden webOS pointer WebSocket recovery”) were also merged into fork `main`.
- Fork PR #4 (native LG MUTE key) and PR #5 (GitHub Actions build/signing-key cutover preparation) were open at the last status check. Verify before acting.
- The dedicated handoff branch is `docs/chat-handoff`. Keep this document updated on that branch, not on feature branches or upstream PR branches.

## Pointer WebSocket investigation — two distinct approaches

The two investigation branches are not duplicate implementations. They represent successive approaches to the same observed failure: navigation initially worked, but after an idle period the TV could stop responding to pointer-button commands while other integration functions remained available.

### Earlier approach: `investigation/pointer-websocket-reliability`

Purpose: detect a pointer connection that appears alive locally but is no longer useful, then discard and reacquire it.

Work included:
- Idle Ping/Pong probing (after approximately five seconds idle, with a bounded response wait).
- Dropping and reacquiring the pointer socket when the probe failed.
- Diagnostics to record socket acquisition, probes, sends, and failures.

Key finding:
- A successful Ping/Pong and successful local WebSocket write did not prove that the LG TV was processing navigation commands. Real-device testing produced false-positive health checks: the probe and local send could succeed while navigation was ignored.
- This approach was useful diagnostically but did not solve the underlying idle servicing problem.

### Replacement approach: `investigation/pointer-websocket-worker-phase1`

Purpose: continuously service the pointer WebSocket instead of periodically probing a socket that is otherwise left unread.

Production implementation:
- Added `src/pointer_worker.rs`.
- A dedicated worker thread exclusively owns and continuously reads the pointer WebSocket, including while idle.
- The worker services Ping/Pong/Close frames and accepts button commands through a bounded command channel.
- The main thread retains ownership of the synchronous Couch device client and SSAP connection. Do not share the synchronous `DeviceClient` across threads.
- Button delivery is not blindly replayed when the result is uncertain. A command known not to have been accepted can be retried after reconnect; an ambiguous transport result is returned as an error without replay.
- Worker shutdown and transport-loss paths are handled explicitly.
- `src/pointer_diagnostics.rs` was synchronized for writes from both the main and worker threads.

This worker implementation supersedes the periodic idle-probe design. Do not combine both approaches or reintroduce the old probe as a parallel health mechanism without new evidence.

### Merge and CI evidence

- Source branch: `investigation/pointer-websocket-worker-phase1`
- Source head before merge: `529c94093c0a6bae37c90f3fb5741ce7bd41a509`
- Fork PR #11: https://github.com/joemacjr92/couch-integration-webos/pull/11
- PR #11 was squash-merged into the user's fork `main` on 2026-11-25.
- CI run #83: https://github.com/joemacjr92/couch-integration-webos/actions/runs/45682505593
- Verified on that run: `cargo fmt -- --check`, `cargo test --locked --all-targets`, pointer-worker tests, signed feed-history validation, and repository public-key validation all passed.
- ARMv7 build was skipped by that workflow. Do not describe this run as an ARMv7 build or package validation.
- No upstream merge occurred. PR #11 targeted the user's fork's `main`, not Couch-OS upstream.

## Physical LG TV validation of the worker

The user deployed the worker implementation and supplied the following evidence:

- Initial ENTER presses successfully acquired the pointer socket, connected the worker, and sent button frames.
- During an approximately ten-minute idle period, the worker continued receiving peer Ping frames. There were no logged reconnects, worker restarts, or transport errors.
- The first RIGHT press after that idle period was reported by the user as registering on the TV.
- Sustained RIGHT, LEFT, and ENTER presses subsequently worked.

This is strong evidence that continuous socket servicing addresses the previously observed idle-navigation failure. It is not a universal compatibility claim or a completed long-duration soak test. A 30-minute idle test followed by a first navigation press was suggested as additional confidence, but no result for that longer test is recorded here.

## Pointer diagnostics and device access

- Diagnostics are written to `/tmp/couch-webos-pointer.log` on the CouchOS device.
- The log is bounded (64 KiB), timestamped, reset on integration process start, and does not record credentials or command payloads.
- The device's `/tmp` is ephemeral tmpfs. The log will not survive a reboot/process lifecycle as durable storage.
- CouchOS core discards plugin stderr and reserves stdout for protocol traffic. Do not use `println!` or `eprintln!` as a diagnostic channel.
- The user's test remote was accessed over SSH at `10.0.0.25` using their existing local key. Do not assume this address applies to another environment.
- Example log commands used by the user:
  - `ssh -i ~/.ssh/id_ed25519_ha100 root@10.0.0.25 'tail -n 50 -f /tmp/couch-webos-pointer.log'`
  - `ssh -i ~/.ssh/id_ed25519_ha100 root@10.0.0.25 'rm -f /tmp/couch-webos-pointer.log'`
- Diagnostics were left enabled in the implementation at the time of this handoff. A default-off setting/toggle was discussed but not implemented. The plugin manifest's existing settings schema did not expose an environment-variable mechanism; inspect the SDK's actual settings schema before proposing a toggle.

## Long-press / repeat investigation — separate from pointer idle recovery

The user reported an error on the remote display when holding RIGHT for fast-forward in an app, although commands continued working afterward. The user suspects the error may originate in CouchOS rather than the webOS integration.

Evidence from the CouchOS code inspected:
- `ui/couch-gui/src/keypad.rs` generates software key-repeat events: repeat begins after roughly 400 ms and repeats at roughly 70 ms intervals.
- `ui/couch-gui/src/tv_plugin.rs` sends a key request for each Tap/Repeat event, then performs status and, where supported, Inputs and Apps requests before rebuilding the UI state.
- `daemon/couch-confd/src/plugins.rs` serializes requests per plugin connection and uses a 250 ms lock-wait allowance before returning a Busy error.
- These code paths make request contention a plausible area to investigate, but do not prove the cause of the remote-screen error.
- The user's captured webOS pointer log showed each observed RIGHT/LEFT/ENTER event reaching the integration and each corresponding worker send succeeding. No worker restart, pointer reconnect, or transport error was recorded during that trace.
- Therefore, the captured integration log does not show an integration-level failure. It does not prove every frame was processed by the TV.
- The user chose to raise the issue with the CouchOS developer first. Do not change webOS integration timing or add throttling/retry behavior based only on this symptom. Revisit only if CouchOS investigation identifies an integration-side requirement or new logs provide evidence.

## Upstream contribution plan for the WebSocket worker

The worker implementation does not inherently depend on upstream PR #6. Do not stack the WebSocket contribution on PR #6 or wait for PR #6 to merge solely because the work was developed on a fork branch that already contained other changes.

However, do not open an upstream PR directly from `investigation/pointer-websocket-worker-phase1`. That branch carries substantial fork history and unrelated tree changes. GitHub comparisons showed it was far ahead of upstream and would produce a noisy PR.

Agreed direction:
1. Start a new, clean contribution branch from the current `Couch-OS/couch-integration-webos:main`.
2. Port only the pointer-worker implementation and the required pointer integration changes.
3. Include only supporting diagnostics if they are justified and suitable for upstream; reconsider always-on diagnostic logging before submission.
4. Carefully reconcile `src/lib.rs` with upstream's current version. Do not copy the entire investigation-branch file over upstream because it includes unrelated changes, including code associated with the separate functional/CEC work.
5. Keep the final diff limited to the worker, necessary pointer lifecycle/button routing, tests, and any justified documentation.
6. Run formatting, locked all-target tests, and the relevant admission/package checks. State explicitly whether ARMv7 was built or skipped.
7. Open a separate PR targeting upstream `main`. PR #6 remains independent and untouched.

Before creating the clean branch or porting code, inspect the exact commits and three-way file differences between upstream `main`, the PR #6 contribution branch, and the worker branch. In particular, inspect overlapping `src/lib.rs` hunks rather than assuming the worker changes are cleanly cherry-pickable.

## Branch distinctions at a glance

| Branch | Role | Current interpretation |
|---|---|---|
| `investigation/pointer-websocket-reliability` | Initial idle Ping/Pong detection and recovery experiment | Historical approach; probe could give false confidence |
| `investigation/pointer-websocket-worker-phase1` | Dedicated continuously-reading pointer WebSocket worker | Production implementation; merged into fork `main` via PR #11 |
| `upstream/webos-functional-improvements` | Separate CEC/audio/functional contribution | Head for upstream PR #6; not a prerequisite for the worker |
| `docs/chat-handoff` | Continuity documentation | Update this branch for future-session handoffs |

## Updated first checks for the next session

1. Read this entire handoff, including this addendum.
2. Check the live fork `main`, upstream `main`, and PR #6 state.
3. Confirm PR #11 remains merged into the fork and inspect the current worker implementation on fork `main`.
4. Check whether the user completed the longer idle soak test; do not assume it happened.
5. If preparing an upstream worker PR, begin from upstream `main` and port only the WebSocket worker changes. Do not use the investigation branch as a direct PR head.
6. Keep the long-press/CouchOS issue separate unless new evidence connects it to the webOS integration.


---

# LG webOS Device UI, Activity Pages, and Future Media Support — 2026-11-25

This section records the investigation into why the LG webOS integration presents differently when opened as a Device versus when used as an Activity, the comparison with the legacy CoreELEC/Kodi path, and the feasibility of adding media metadata/artwork support. These findings were made by inspecting CouchOS core and integration source. Re-check the live source before implementation because core behavior and protocol support may change.

## Device screen versus Activity screen

### Observed behavior

- Opening the LG webOS integration directly as a Device presents the native Couch television-style screen: a prominent TV/media area with artwork-oriented presentation and transport controls, rather than a collection of package command pages.
- Selecting the same LG integration as an Activity source presents the package-defined Activity pages: navigation, playback/transport commands, input selection, and other declared controls.
- This is not simply a setting that switches between two layouts. The Device and Activity entry points take different routes through CouchOS core.

### CouchOS source-path findings

The inspected core files were:
- `ui/couch-gui/src/shortcuts.rs`
- `ui/couch-gui/src/activity.rs`
- `ui/couch-gui/src/activity_pages.rs`
- `ui/couch-gui/ui/screens/tv.slint`

The relevant behavior is:

1. The Device shortcut path uses `opens_player(integration)` for built-in player integrations. That helper recognizes `Integration::Kodi` and `Integration::Sonos`; it does not classify `Integration::Plugin` as a built-in player.
2. For a packaged television, the Device path can instead recognize the native TV connection/profile and open the Couch native TV screen.
3. The Activity controller resolves a plugin-backed Activity source through `plugin_target(...)`. When it finds an `Integration::Plugin`, it opens the plugin Activity page renderer and returns before reaching the generic/custom Activity page path.
4. The plugin page renderer builds pages from the package's declared presentation components. It does not consult the Device screen's native-TV profile to switch the Activity to the native TV screen.

This explains the difference: the same packaged LG integration can qualify for the native TV Device screen while its Activity still uses the generic package presentation renderer.

### Native packaged-TV behavior and core PR #338

CouchOS core PR [#338 — Restore native packaged TV controls](https://github.com/Couch-OS/couch/pull/338) was merged on 2026-09-24 (merge commit `24f808afe0e04e9dd0b32a15e510ccf3222f3437`).

The inspected change recognizes a packaged television from its existing input selector, `supports_inputs`, and complete standard TV capability set. It restores the native Couch TV hero and transport row without adding a vendor-specific manifest flag or changing the package protocol. It also routes the expected physical TV keys through the package. Incomplete TV/receiver capability sets continue to use the generic tile navigator.

The important boundary is that this native packaged-TV recognition explains the Device presentation; the Activity plugin path described above does not currently reuse that native screen.

The webOS package already declares `supports_inputs: true` and an `input_selector` presentation component. Its input implementation uses LG SSAP to retrieve the external input list and switch inputs.

## Why the Activity has multiple pages

The inspected `ui/couch-gui/src/activity_pages.rs` implementation maps each declared presentation component to its own page. This includes components such as:
- `InputSelector`
- `CommandGroup`
- `StatusText`
- `Toggle`
- `VolumeDbControl`
- `SoundOutputSelector`

The input selector is not missing or unsupported. It is deliberately rendered as its own page by the current plugin Activity renderer. There is no current manifest layout option that combines the input selector with the Navigation or Playback command groups on one page.

**Decision:** Keep the input selector in the manifest. Removing it merely to reduce page count would remove useful functionality and would not create a combined page. Combining components or arranging multiple components on one Activity page requires a CouchOS core renderer/protocol enhancement; it cannot be achieved by changing only the webOS integration manifest.

## Comparison with legacy CoreELEC / Kodi

The old CoreELEC/Kodi path differs materially from the packaged Kodi integration:

- In `model/couch-model/src/connection.rs`, both legacy `Provider::Kodi` and `Provider::CoreElec` connections resolve to the built-in `Integration::Kodi`.
- The Device shortcut logic recognizes `Integration::Kodi` as a built-in player and routes it to CouchOS's native Kodi player controller.
- CoreELEC's SSH/system-management functionality is separate from the Kodi media-control/player path.
- The newer packaged Kodi integration is a plugin. Its README describes it as a preview and explicitly says it does not replace the built-in Kodi integration or provide the full native player screen. Its package presentation is currently a generic set of command/control pages.

The legacy CoreELEC experience therefore benefits from CouchOS's built-in Kodi player implementation; it is not evidence that a generic package Activity can currently render a native player screen. The packaged LG integration's native Device screen is a separate packaged-TV capability introduced by the core native-TV work, while its Activity remains package-page based.

Reference: [CouchOS Kodi integration README](https://github.com/Couch-OS/couch-integration-kodi/blob/main/README.md).

## Media player and artwork feasibility

### CouchOS support: design/model exists, runtime path is not complete

The following core files/plans were inspected:
- `docs/plans/protocol-3-t4-media.md`
- `docs/plans/media-player-component-design.md`
- `clients/couch-plugin/src/manifest.rs`
- `model/couch-model/src/connection.rs`
- `ui/couch-gui/src/activity_pages.rs`

There are media-player model/design types, including concepts for layout, artwork, lists, up-next content, navigation, refresh intervals, and keys. The approved T4 media plan describes a package-provided media component with Couch-rendered native player UI, metadata, artwork, seek/transport, lists, and selection operations.

However, this is not currently a usable package feature in the inspected source:

- The manifest validator explicitly rejects `PluginComponent::MediaPlayer` (and `VolumePercentControl`) pending the required protocol implementation.
- The Activity page renderer explicitly ignores `MediaPlayer` components.
- The design document is a proposal/design reference, not proof of a working end-to-end implementation.
- The media plan is marked approved but describes work that still needs to land. Do not add a `media_player` component to the webOS manifest and assume CouchOS will consume it.

The media plan describes artwork transfer constraints (including bounded size/time and supported image MIME types). The inspected GUI image decoder path appeared to support JPEG/PNG but not WebP; verify current decoder support before relying on the plan's broader MIME list.

**Conclusion:** A CouchOS core implementation is a prerequisite for a real package-provided media player. The webOS integration alone cannot make artwork appear in the native Couch player.

### LG webOS metadata/artwork API: promising, not yet proven on retail TVs

The official webOS OSE Media Controller API documentation describes the Luna service `com.webos.service.mediacontroller`, including:
- Media session discovery and session information.
- Playback status and position.
- Media metadata such as title, artist, album, and duration.
- A `coverArt` value containing a source URL.

Reference: [LG webOS OSE Media Controller API](https://www.webosose.org/docs/reference/ls2-api/com-webos-service-mediacontroller/).

Important limitation: this is a Luna Service API documented for webOS Open Source Edition. It is not an SSAP endpoint already exposed through the integration's existing remote WebSocket connection. The investigation did not establish that the service is available to third-party apps, or remotely accessible, on retail LG webOS televisions. Do not assume the OSE API can be called over SSAP or that its cover-art URL is reachable from CouchOS.

The current webOS integration does not retrieve media-session metadata or artwork. Its status path currently obtains power, volume, foreground app information, and optional sound/picture information over SSAP. The foreground app title is not equivalent to the active media session's title/artist/album/artwork.

### Recommended proof-of-concept before implementation

When core media-player support is available, first establish LG-side feasibility on the actual target TV:

1. Determine whether the TV exposes a supported media-session API to a third-party app or remote client.
2. Verify whether the relevant Luna service is available on the retail TV and what permissions/service-call context it requires.
3. Determine whether the service can be accessed from the integration's existing SSAP connection. If not, identify a supported, secure local access path rather than assuming SSAP can proxy Luna calls.
4. Test with multiple foreground apps and media types; establish when metadata and artwork are present, absent, delayed, or stale.
5. Verify that the returned artwork URL is reachable from CouchOS, including any authentication, local-only URL, certificate, or lifetime constraints.
6. Only then design the package-side metadata/artwork reporting and map it to the CouchOS media protocol that has actually landed.

Do not hard-code assumptions about one streaming app's metadata behavior as a general webOS capability.

## Agreed direction and sequencing

The current direction is to avoid speculative changes and revisit this after the outstanding upstream contributions have had a chance to land.

Recommended sequence:

1. **Wait for the two upstream contribution tracks to resolve.** Check live upstream PR/branch state before relying on any PR number or claiming either has landed. The functional CEC/audio contribution is upstream PR #6. The pointer-worker work is a separate contribution and must remain separate; use the clean upstream-based branch/PR rather than the fork investigation branch.
2. **Re-check CouchOS media-player implementation.** Confirm the manifest validator accepts the component, protocol requests are implemented, and the GUI actually renders and services it. The current source inspected here does not satisfy those conditions.
3. **Run the LG media-session proof-of-concept.** Establish actual retail-TV API availability, metadata quality, and artwork URL accessibility before implementing integration support.
4. **Implement package media reporting only against a landed, supported CouchOS contract.** Keep the webOS integration's current status/input/command behavior working.
5. **Revisit Activity page composition separately.** Determine whether CouchOS has added a way to compose components on one page or whether a core change is still needed. Keep the input selector unless a better native input-selection path replaces it.

Do not block the LG API proof-of-concept on Activity layout work; they are separate investigations. Media player/artwork is the larger potential enhancement, while combining Activity controls is a CouchOS UI/layout question.

## Key cautions for future sessions

- Do not repeat the earlier incorrect explanation that a setting such as “Open custom pages first” alone switches a packaged LG Activity between the generic pages and the native TV Device screen. The source paths are distinct.
- Do not equate legacy CoreELEC/Kodi with the packaged Kodi plugin. Legacy CoreELEC resolves to built-in Kodi; the package is a plugin with a different UI path.
- Do not remove the webOS input selector to work around page count.
- Do not claim CouchOS package media-player support is implemented merely because model types and design documents exist.
- Do not claim the OSE Luna Media Controller API is remotely accessible on retail LG TVs until verified on hardware.
- Keep all proposed CouchOS renderer/protocol changes clearly identified as core work, not webOS integration-only changes.
