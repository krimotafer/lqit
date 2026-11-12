#!/usr/bin/env bash
# End-to-end benchmarks with hyperfine: minigrep vs GNU grep vs ripgrep.
# Run it from the project root: ./scripts/bench_cli.sh
set -euo pipefail

command -v hyperfine >/dev/null || { echo "hyperfine not found (cargo install hyperfine)" >&2; exit 1; }

CORPUS="data/pg100.txt"
[ -f "$CORPUS" ] || { echo "$CORPUS not found, run the script from the project root" >&2; exit 1; }

OUT="target/benchmarks"
mkdir -p "$OUT"

cargo build --release --quiet
MINIGREP="./target/release/minigrep"

# WARN: '--output=pipe' is important. By default hyperfine sends the output to /dev/null,
# GNU grep detects that and stops at the first match (like -q). So grep looks much faster
# than it really is. With a pipe every tool has to write all the matching lines.
# '-N' runs the commands without a shell, less noise when a run takes ~2 ms.
# '-i' because grep and rg exit with 1 when nothing match (pure miss scenario).
bench() {
    local name=$1
    local args=$2
    local cmds=("$MINIGREP $args $CORPUS" "grep $args $CORPUS")
    if command -v rg >/dev/null; then
        cmds+=("rg $args $CORPUS")
    fi

    echo ""
    echo "== $name: $args"
    hyperfine -N --output=pipe -i --warmup 5 --runs 50 \
        --export-markdown "$OUT/$name.md" \
        --export-json "$OUT/$name.json" \
        "${cmds[@]}"
}

bench raw_sparse     "Hamlet"      # few matches (105 lines)
bench raw_dense      "the"         # a lot of matches (39,634 lines), stress the printer
bench flagged_sparse "-in Hamlet"  # -i and -n together (469 lines)
bench pure_miss      "zzzzzzzz"    # no match, only the cost of reading the file line by line

echo ""
echo "Done, results in $OUT/"
