# Couch LG webOS integration

This repository is the extraction candidate for Couch's built-in LG webOS
client. It owns the TV's SSAP WebSocket, on-screen approval pairing, client key
and pinned certificate, navigation, playback, volume, mute, status and input
selection. Couch keeps the native screen and the integration host.

`0.1.0_pre5` restores the native Apps, Picture and Sound output cards. It uses
protocol 5 to enumerate installed launch points, launch an app, report the
current sound output and expose Couch-owned sound choices. The Picture card
opens the TV's own Settings app, matching the original built-in screen.

`0.1.0_pre4` accepts the resource path in the pointer WebSocket URL returned by
the TV, restoring D-pad and other pointer-backed controls. The setup field
remains restricted to the TV's IP address or a root WebSocket URL.

`0.1.0_pre3` also reports the foreground app's human name when the TV includes
it in status. A core containing the packaged-TV touchscreen update recognizes this
package's existing input selector and complete standard TV capability set,
restores Couch's TV hero and transport row, and routes the physical D-pad to
the television. That release remained compatible with protocol-3 cores, which
kept the generic package screen until the core was updated. `0.1.0_pre5`
requires protocol 5 because app discovery is a new host/package exchange.

Fork builds use an APK-compatible version suffix so they sort after the matching
upstream prerelease without colliding with its package identity. For example,
the fork build based on upstream `0.1.0_pre5` is packaged as
`0.1.0_pre5_git1`. Use this exact canonical version in the workflow input,
plugin manifest, APK filename, feed metadata and GitHub Release tag
(`v0.1.0_pre5_git1`). Do not use a dotted label such as
`0.1.0_pre5.fork1` as the package version; it is not valid Alpine APK
version syntax.

The package is intentionally preview-only. Network power-on remains outside the
current package boundary because it needs a host-learned Wake-on-LAN address or
the remote's privileged infrared device. This package therefore offers network
power-off only.

The only public setting is the TV IP address. The package uses encrypted webOS
control on port 3001 automatically. Full `ws://` and `wss://` URLs saved by the
first preview remain accepted when that package is upgraded. Pairing asks the
owner to approve Couch on the TV. The client key and exact TLS certificate are
returned as a protocol-3 credential; they never enter settings, argv, the
environment or exported Couch configuration.

## HDMI-CEC inputs

The input selector keeps the physical inputs reported by
`ssap://tv/getExternalInputList` and augments them with downstream devices
reported by `ssap://com.webos.service.eim/getAllInputStatus`. In the EIM
response, `id` identifies the physical input (for example, `HDMI_2`), while
`appId` identifies the parent HDMI launch-point app (for example,
`com.webos.app.hdmi2`). The `subList` contains that input's CEC descendants.
The parser also accepts the alternate `inputId` field when present.

CEC descendants are flattened into the existing Couch input selector and
labelled with their parent input's current display name (for example,
`Blu-ray - p3i`). If multiple physical inputs share the same display name,
the HDMI port is included to keep their descendants distinguishable. When a
friendly parent name is unavailable, the HDMI port label is used as a fallback.
The top-level input list is retained unchanged. A depth-one CEC entry whose
TV-reported name already appears in the top-level input list is not added a
second time; this comparison is based on the current TV probe, not a
hardcoded device or brand name. Where CEC vendor ID and physical address are
available, those are also used to collapse duplicate CEC reports.

Selecting a physical input continues to call `ssap://tv/switchInput`.
Selecting a CEC descendant verifies that its `uniqueId` is still present,
launches the parent HDMI app through
`ssap://com.webos.applicationManager/launch` with the original
`uniqueId` parameter, waits 2.5 seconds, and verifies EIM's
`lastUniqueId`. A failed or stale selection is not blindly retried.

CEC discovery is best-effort: if EIM status cannot be read, the physical
input list remains available. The selector is refreshed when Couch requests
its inputs; this package does not add background polling.

## Audio status subscriptions

The integration subscribes to `ssap://audio/getStatus` for mute state and
`ssap://audio/getVolume` for volume percentage. Subscription notifications
are correlated by their SSAP message IDs and processed during normal
synchronous WebSocket reads. The mute toggle uses the latest valid mute state
observed on that connection; successful mute commands keep the local cache
consistent.

These subscriptions are best-effort and do not create a background receiver.
Notifications are not consumed while the package is idle, and the integration
cannot independently push status changes into Couch. See
[`docs/audio-subscriptions.md`](docs/audio-subscriptions.md) for the protocol,
cache behavior, and verification details.

## Build and test

```sh
cargo fmt -- --check
cargo test --locked --all-targets
cargo build --locked --release --bin couch-plugin-webos
```

No test or build command pairs with a physical TV. Hardware validation must be
scheduled separately and should cover a secure pairing, reconnect, all remote
buttons, status and input selection, network power-off, core-owned wake/IR power,
package upgrade and rollback.

## Preview package feed

The webOS integration uses the CouchOS ARMv7 APK packaging and signed preview
feed format. Once the first package is explicitly published, the feed will be
served from:

`https://joemacjr92.github.io/couch-integration-webos/preview`

The repository layout on the `gh-pages` branch is:

```
.nojekyll
preview/
└── armv7/
    ├── APKINDEX.tar.gz
    ├── couch-integration-webos-<version>-r0.apk
    ├── feed.json
    └── feed.json.sig
```

The committed repository verification key is
[`keys/webos-repository-public.pem`](keys/webos-repository-public.pem). The
matching private key and public-key secret are held in the repository's GitHub
Actions secrets as `APK_PRIVATE_KEY` and `APK_PUBLIC_KEY`; the private key
must never be committed. The packaging workflow checks that the private key,
public-key secret and committed public key all match before signing anything.

The committed public-key SHA-256 fingerprint is `499b0a9cfa8de88136f1f608221543eadfb43bea86af2782ed940013a11b332b`.

### Building and publishing

Every push to `feature/hdmi-cec-inputs` automatically runs a
**forced dry-run build** using the canonical package version
`0.1.0_pre5_git1`. This exercises the ARMv7 build, APK signing, repository
index generation and feed validation. It uploads the APK and signed preview
feed as the `webos-package-feed` workflow artifact; it cannot publish to
GitHub Pages or create a GitHub Release.

After this workflow has been merged into the default branch, use
**Actions → Build webOS integration package → Run workflow** to dispatch a
build manually and provide the desired APK-compatible version.

- Leave **dry_run** enabled to build and validate without publishing. The
  signed APK and preview feed are uploaded as a workflow artifact.
- Publishing is allowed only for an explicit workflow dispatch from `main`
  with dry-run disabled. A separate, gated publish job downloads the validated
  artifact, updates the `gh-pages` branch and creates a GitHub Release.
- The package build job has only `contents: read` permission. Only the gated
  publish job receives `contents: write`; the weekly feed-renewal job scopes
  write permission to its main-branch job.
- The initial publication creates the feed branch if it does not exist.
  Existing feed history is signature-checked, hash-checked and version-checked
  before it is carried forward.
- A separate weekly workflow renews the signed feed metadata expiry without
  rebuilding or changing the published APKs.

After the first publication, enable GitHub Pages in repository settings with
the `gh-pages` branch and repository root as the publishing source. The feed
is not live until that first publication and Pages configuration are complete.
