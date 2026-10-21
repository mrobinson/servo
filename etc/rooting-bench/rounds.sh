#!/bin/bash
# Run the given harness mode for each variant, alternating variants in rounds.
# Usage: rounds.sh MODE OUTDIR ROUNDS VARIANT... [-- extra harness args]
# Each variant is a binary at $BIN/servoshell-VARIANT (default: target/release of this checkout).
set -e
MODE=$1; OUT=$2; ROUNDS=$3; shift 3
VARIANTS=()
while [ $# -gt 0 ] && [ "$1" != "--" ]; do VARIANTS+=("$1"); shift; done
[ "$1" = "--" ] && shift
HERE=$(dirname "$0")
BIN=${BIN:-$HERE/../../target/release}
mkdir -p "$OUT"
for round in $(seq 1 "$ROUNDS"); do
  for variant in "${VARIANTS[@]}"; do
    python3 -I "$HERE/harness.py" "$MODE" "$BIN/servoshell-$variant" "$OUT/$variant-$round.json" "$@"
    echo "round $round $variant done"
  done
done
