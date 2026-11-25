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
