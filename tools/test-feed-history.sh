#!/usr/bin/env bash
# Regression tests for signed webOS preview feed history validation.
set -Eeuo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
VALIDATOR="$ROOT/tools/validate-feed-history.sh"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
for tool in jq openssl sha256sum; do command -v "$tool" >/dev/null; done
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "$WORK/private.pem" >/dev/null 2>&1
openssl pkey -in "$WORK/private.pem" -pubout -out "$WORK/public.pem" >/dev/null
export RUNNER_TEMP="$WORK"
fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }
make_fixture() {
  local dir="$WORK/feed"; rm -rf "$dir"; mkdir -p "$dir"
  printf 'test APKINDEX bytes\n' > "$dir/APKINDEX.tar.gz"
  printf 'test webOS APK bytes\n' > "$dir/couch-integration-webos-0.1.0_pre5-r0.apk"
  local is ih as ah
  is="$(stat -c '%s' "$dir/APKINDEX.tar.gz")"; ih="$(sha256sum "$dir/APKINDEX.tar.gz" | awk '{print $1}')"
  as="$(stat -c '%s' "$dir/couch-integration-webos-0.1.0_pre5-r0.apk")"; ah="$(sha256sum "$dir/couch-integration-webos-0.1.0_pre5-r0.apk" | awk '{print $1}')"
  jq -n --argjson is "$is" --arg ih "$ih" --argjson as "$as" --arg ah "$ah" '
    {schema:1,channel:"preview",sequence:2,index:{path:"APKINDEX.tar.gz",size:$is,sha256:$ih},
     packages:[{id:"webos",version:"0.1.0_pre5",apk:"couch-integration-webos-0.1.0_pre5-r0.apk",
     size:$as,sha256:$ah,protocol_version:5,min_core_protocol_version:5}]}
  ' > "$dir/feed.json"
  openssl dgst -sha256 -sign "$WORK/private.pem" -out "$dir/feed.json.sig" "$dir/feed.json"
}
sign_feed() { openssl dgst -sha256 -sign "$WORK/private.pem" -out "$WORK/feed/feed.json.sig" "$WORK/feed/feed.json"; }
run_ok() { bash "$VALIDATOR" "$WORK/feed" "$WORK/public.pem" "$1" "${@:2}" >/dev/null || fail "expected success for $1"; }
run_bad() { if bash "$VALIDATOR" "$WORK/feed" "$WORK/public.pem" "$1" "${@:2}" >/dev/null 2>&1; then fail "expected rejection for $1"; fi; }
make_fixture; run_ok 0.1.0_pre6; pass "valid signed prerelease feed and newer prerelease"
run_ok 0.1.0_pre5 --allow-existing-version; pass "existing version allowed only for explicit dry run"
run_bad 0.1.0_pre5; pass "existing version rejected for publication"
run_bad 0.1.0_pre4; pass "older prerelease rejected"
make_fixture; printf 'tamper' >> "$WORK/feed/feed.json"; run_bad 0.1.0_pre6; pass "invalid feed signature rejected"
make_fixture; printf 'tamper' >> "$WORK/feed/APKINDEX.tar.gz"; run_bad 0.1.0_pre6; pass "APKINDEX size/hash mismatch rejected"
make_fixture; printf 'tamper' >> "$WORK/feed/couch-integration-webos-0.1.0_pre5-r0.apk"; run_bad 0.1.0_pre6; pass "historical APK hash mismatch rejected"
make_fixture; rm "$WORK/feed/couch-integration-webos-0.1.0_pre5-r0.apk"; run_bad 0.1.0_pre6; pass "missing historical APK rejected"
make_fixture; jq '.packages += .packages' "$WORK/feed/feed.json" > "$WORK/feed/modified.json"; mv "$WORK/feed/modified.json" "$WORK/feed/feed.json"; sign_feed; run_bad 0.1.0_pre6; pass "duplicate package/version rejected"
make_fixture; jq '.packages[0].apk = "couch-integration-webos-0.1.0_pre4-r0.apk"' "$WORK/feed/feed.json" > "$WORK/feed/modified.json"; mv "$WORK/feed/modified.json" "$WORK/feed/feed.json"; sign_feed; run_bad 0.1.0_pre6; pass "filename/version mismatch rejected"
make_fixture; jq '.sequence = 1.5' "$WORK/feed/feed.json" > "$WORK/feed/modified.json"; mv "$WORK/feed/modified.json" "$WORK/feed/feed.json"; sign_feed; run_bad 0.1.0_pre6; pass "non-integer sequence rejected"
make_fixture; jq '.packages[0].protocol_version = 3' "$WORK/feed/feed.json" > "$WORK/feed/modified.json"; mv "$WORK/feed/modified.json" "$WORK/feed/feed.json"; sign_feed; run_bad 0.1.0_pre6; pass "wrong Couch protocol version rejected"
echo "All webOS feed-history validation tests passed."
