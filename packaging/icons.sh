#!/usr/bin/env bash
# Regenerate every app icon from assets/app-icon/yedit.svg (the canonical master).
#
# Needs: resvg (brew install resvg / cargo install resvg). On macOS, iconutil also writes the
# .icns. The outputs are committed, so packaging never needs these tools.
#
#   packaging/icons.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/assets/app-icon"
SVG="$DIR/yedit.svg"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

command -v resvg >/dev/null || { echo "error: resvg not found (brew install resvg)" >&2; exit 1; }

# The artwork is a full-bleed 512-unit tile (rx=112). macOS icons pad it to Apple's 824/1024 body
# grid (transparent margin). Windows and Linux icons crop 22 units off each side (into the
# rounded corners) so the portrait reads at 16-48 px.
grep -q 'viewBox="0 0 512 512"' "$SVG" || { echo "error: expected viewBox=\"0 0 512 512\" in $SVG" >&2; exit 1; }
MAC="$TMP/mac.svg"
sed 's/viewBox="0 0 512 512"/viewBox="-62 -62 636 636"/' "$SVG" >"$MAC"
TIGHT="$TMP/tight.svg"
sed 's/viewBox="0 0 512 512"/viewBox="22 22 468 468"/' "$SVG" >"$TIGHT"

render() { resvg -w "$2" -h "$2" "$1" "$3" </dev/null; }

render "$MAC" 1024 "$DIR/yedit-1024.png"

# Linux hicolor theme.
for s in 16 24 32 48 64 128 256 512; do
  mkdir -p "$DIR/hicolor/${s}x${s}/apps"
  render "$TIGHT" "$s" "$DIR/hicolor/${s}x${s}/apps/com.fesilabs.yedit.png"
done
mkdir -p "$DIR/hicolor/scalable/apps"
# The lighter trace (yedit-small.svg) keeps the scalable theme icon cheap to render.
cp "$DIR/yedit-small.svg" "$DIR/hicolor/scalable/apps/com.fesilabs.yedit.svg"

# Windows .ico.
ICO_PNGS=()
for s in 16 20 24 32 40 48 64 128 256; do
  render "$TIGHT" "$s" "$TMP/ico-$s.png"
  ICO_PNGS+=("$TMP/ico-$s.png")
done
(cd "$ROOT" && cargo run -q -p xtask -- ico "$DIR/yedit.ico" "${ICO_PNGS[@]}")

# macOS .icns.
if command -v iconutil >/dev/null; then
  SET="$TMP/yedit.iconset"
  mkdir -p "$SET"
  for s in 16 32 128 256 512; do
    render "$MAC" "$s" "$SET/icon_${s}x${s}.png"
    render "$MAC" $((s * 2)) "$SET/icon_${s}x${s}@2x.png"
  done
  iconutil -c icns -o "$DIR/yedit.icns" "$SET"
else
  # No iconutil (Windows, Linux): pack PNG-compressed entries directly, the types iconutil
  # writes for an iconset (16..512 at 1x and 2x).
  for s in 16 32 64 128 256 512 1024; do
    render "$MAC" "$s" "$TMP/icns-$s.png"
  done
  perl -e '
    my @entries = (["icp4", 16], ["ic11", 32], ["icp5", 32], ["ic12", 64], ["ic07", 128],
                   ["ic13", 256], ["ic08", 256], ["ic14", 512], ["ic09", 512], ["ic10", 1024]);
    my $body = "";
    for my $e (@entries) {
      my ($type, $size) = @$e;
      open my $in, "<:raw", "$ARGV[0]/icns-$size.png" or die "icns-$size.png: $!";
      my $png = do { local $/; <$in> };
      $body .= $type . pack("N", 8 + length $png) . $png;
    }
    open my $out, ">:raw", $ARGV[1] or die "$ARGV[1]: $!";
    print $out "icns", pack("N", 8 + length $body), $body;
  ' "$TMP" "$DIR/yedit.icns"
fi
echo "icons written to $DIR"
