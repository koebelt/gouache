#!/usr/bin/env bash
# Phase 0.4 black-box capture helper (macOS). Private reference material: output is git-ignored, never shipped.
# Usage: scripts/capture.sh <label> [frames=1] [interval_s=0.5] [note...]
#   e.g. scripts/capture.sh tool_drag_wall 24 0.25 "drag a wall left to right"
#        scripts/capture.sh sweep_autumn_noon_high 1 0 "autumn, noon, high camera"
# Requires: Screen Recording permission for your terminal (System Settings > Privacy & Security).
set -euo pipefail
label="${1:?label required}"; frames="${2:-1}"; interval="${3:-0.5}"; shift $(( $# < 3 ? $# : 3 )) || true
note="${*:-}"
root="$(cd "$(dirname "$0")/.." && pwd)/docs/reference/captured"
dir="$root/$label"; mkdir -p "$dir"
echo "Capturing $frames frame(s) of '$label' in 3 s — switch to Tiny Glade..."; sleep 3
for ((i = 0; i < frames; i++)); do
  screencapture -x -m "$(printf '%s/frame_%03d.png' "$dir" "$i")"
  [[ "$i" -lt $((frames - 1)) ]] && sleep "$interval"
done
printf '{"label":"%s","frames":%d,"interval_s":%s,"note":"%s","captured_at":"%s"}\n' \
  "$label" "$frames" "$interval" "$note" "$(date -u +%FT%TZ)" >> "$root/index.jsonl"
echo "Saved to $dir"
