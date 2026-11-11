#!/usr/bin/env bash
set -euo pipefail

# 1. Dependency checks
command -v hyperfine >/dev/null 2>&1 || {
    echo "Error: 'hyperfine' is not installed. Install via: cargo install hyperfine" >&2
    exit 1
}

CORPUS="data/pg100.txt"
if [ ! -f "$CORPUS" ]; then
    echo "Error: Dataset '$CORPUS' not found." >&2
    exit 1
fi

OUT_DIR="target/benchmarks"
mkdir -p "$OUT_DIR"

# 2. Release build
echo "Building minigrep in release mode..."
cargo build --release --quiet

MINIGREP="./target/release/minigrep"

# Detect ripgrep availability
RG_CMD=""
if command -v rg >/dev/null 2>&1; then
    RG_CMD="rg"
fi

echo "Running CLI Benchmarks against Shakespeare dataset (5.64 MB)..."
echo "================================================================="

# -------------------------------------------------------------
# Scenario 1: Raw Exact Search - Sparse Match ("Hamlet")
# Isolates pattern matching and streaming without flag overhead
# -------------------------------------------------------------
echo -e "\n[1/4] Scenario: Raw Exact Search (Sparse match: 'Hamlet')"

hyperfine \
  --warmup 3 \
  --runs 50 \
  --export-markdown "$OUT_DIR/raw_exact_sparse.md" \
  --export-json "$OUT_DIR/raw_exact_sparse.json" \
  --command-name "minigrep (raw)" "$MINIGREP 'Hamlet' $CORPUS" \
  --command-name "grep (raw)"     "grep 'Hamlet' $CORPUS" \
  ${RG_CMD:+--command-name "ripgrep (raw)" "$RG_CMD 'Hamlet' $CORPUS"}

# -------------------------------------------------------------
# Scenario 2: Raw Exact Search - Dense Match ("the")
# Stresses unformatted write throughput across ~28,000 matches
# -------------------------------------------------------------
echo -e "\n[2/4] Scenario: Raw Exact Search (Dense match: 'the')"

hyperfine \
  --warmup 3 \
  --runs 30 \
  --export-markdown "$OUT_DIR/raw_exact_dense.md" \
  --export-json "$OUT_DIR/raw_exact_dense.json" \
  --command-name "minigrep (raw)" "$MINIGREP 'the' $CORPUS" \
  --command-name "grep (raw)"     "grep 'the' $CORPUS" \
  ${RG_CMD:+--command-name "ripgrep (raw)" "$RG_CMD 'the' $CORPUS"}

# -------------------------------------------------------------
# Scenario 3: Flagged Search (-i -n, Sparse Match: "Hamlet")
# Realistic user query with case insensitivity and line numbers
# -------------------------------------------------------------
echo -e "\n[3/4] Scenario: Flagged Search (-i -n, 'Hamlet')"

hyperfine \
  --warmup 3 \
  --runs 50 \
  --export-markdown "$OUT_DIR/flagged_sparse.md" \
  --export-json "$OUT_DIR/flagged_sparse.json" \
  --command-name "minigrep (-in)" "$MINIGREP -in 'Hamlet' $CORPUS" \
  --command-name "grep (-in)"     "grep -in 'Hamlet' $CORPUS" \
  ${RG_CMD:+--command-name "ripgrep (-in)" "$RG_CMD -in 'Hamlet' $CORPUS"}

# -------------------------------------------------------------
# Scenario 4: Pure Scan / Negative Match ("zzzzzzzz")
# Measures baseline I/O throughput and scanning floor
# -------------------------------------------------------------
echo -e "\n[4/4] Scenario: Pure Miss Scan ('zzzzzzzz')"

hyperfine \
  --warmup 3 \
  --runs 50 \
  --export-markdown "$OUT_DIR/pure_miss.md" \
  --export-json "$OUT_DIR/pure_miss.json" \
  --command-name "minigrep" "$MINIGREP 'zzzzzzzz' $CORPUS || true" \
  --command-name "grep"     "grep 'zzzzzzzz' $CORPUS || true" \
  ${RG_CMD:+--command-name "ripgrep" "$RG_CMD 'zzzzzzzz' $CORPUS || true"}

echo -e "\nBenchmarks completed successfully."
echo "Results saved to: $OUT_DIR/ (*.md and *.json)"
