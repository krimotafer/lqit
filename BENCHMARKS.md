# Benchmarks

This is not a race against grep or ripgrep. I bench to understand where the time goes in my code, and to check if my "optimizations" are real or just in my head.

## Setup

- **Dataset**: `data/pg100.txt`, the complete works of Shakespeare (Project Gutenberg). 5.64 MB (5,638,480 bytes), 196,398 lines, ~29 bytes per line, CRLF line endings.
- **Machine**: Intel Core Ultra 7 155H (22 threads, 24 MiB L3 cache), 30 GiB RAM, Linux 7.0.
- **Rust**: rustc 1.98.0, `cargo build --release` (default release profile).
- **Tools**: [Criterion](https://github.com/bheisler/criterion.rs) 0.5 for each component, [hyperfine](https://github.com/sharkdp/hyperfine) 1.18 for the full CLI.
- **Compared with**: GNU grep 3.11 and ripgrep 14.1.0.

How to run:

```bash
cargo bench                        # all Criterion benches (results in target/criterion/)
cargo bench --bench matcher_bench  # only one
./scripts/bench_cli.sh             # full CLI with hyperfine (results in target/benchmarks/)
```

---

## 1. Full CLI (hyperfine)

`scripts/bench_cli.sh`: 5 warmup runs, 50 runs, no shell (`-N`), output sent to a pipe (`--output=pipe`). Measured on 2026-11-12. All three tools print exactly the same lines for each scenario (I checked with `wc -l`).

> **WARN: my first version of this table was wrong.**
> hyperfine sends the output to `/dev/null` by default, and GNU grep detects `/dev/null` and stops at the first match (like `-q`). So grep looked 11x to 19x faster than lqit on the scenarios with matches, but it was not doing the same work. I checked it with `strace`: grep does 7 `read` calls when the output is `/dev/null` and 65 when it is a pipe. Now the script uses `--output=pipe`, so every tool writes all the lines.

| Scenario | Matching lines | lqit | GNU grep | ripgrep |
| :--- | ---: | ---: | ---: | ---: |
| `Hamlet` (few matches) | 105 | 16.4 ± 1.5 ms | 5.9 ± 0.9 ms | 3.1 ± 0.5 ms |
| `the` (a lot of matches) | 39,634 | 20.1 ± 1.3 ms | 11.4 ± 1.3 ms | 7.6 ± 1.0 ms |
| `-in Hamlet` | 469 | 20.4 ± 1.5 ms | 11.9 ± 1.4 ms | 4.5 ± 0.5 ms |
| `zzzzzzzz` (no match) | 0 | 14.5 ± 1.8 ms | 1.4 ± 0.2 ms | 2.6 ± 0.4 ms |

User time (my code) vs system time (kernel):

| Scenario | lqit user / sys | GNU grep user / sys | ripgrep user / sys |
| :--- | :---: | :---: | :---: |
| `Hamlet` | 15.2 / 1.1 ms | 4.6 / 1.2 ms | 1.5 / 1.5 ms |
| `the` | 18.6 / 1.4 ms | 9.9 / 1.4 ms | 5.9 / 1.6 ms |
| `-in Hamlet` | 19.2 / 1.0 ms | 10.4 / 1.4 ms | 2.7 / 1.7 ms |
| `zzzzzzzz` | 13.3 / 1.1 ms | 0.4 / 0.8 ms | 1.3 / 1.3 ms |

What I see:

- System time is ~1 ms for the three tools. So the kernel side (read the file, write the output) is not my problem: `BufReader`, one `stdout().lock()` and `BufWriter` do their job.
- My problem is in user space. Even when nothing matches, lqit needs ~13 ms of CPU, grep needs 0.4 ms. The no-match scenario is the honest "floor": ~10x slower than grep.
- `-i` costs me ~4 ms (20.4 vs 16.4 ms), this is my naive byte loop in `Matcher::CaseInsensitive`.

---

## 2. Components (Criterion)

These numbers are from my `cargo bench` runs on 2026-11-12 (same machine, default Criterion settings).

> **WARN: Criterion has its own trap.** My first printer and searcher benches wrote into `io::sink()`. The compiler can see that sink does nothing, so it deleted most of the work that only exists to produce the output, and the bench measured almost only the loop. Now these benches write into a `Vec<u8>` allocated once, cleared at each iteration (the capacity stays, so no allocation in the timed part) and passed to `black_box` at the end, so the output must really exist. Writing into a `Vec` is a memory copy, close to what `BufWriter` does in `main`.

### Matcher (`benches/matcher_bench.rs`)

The full file as one `&str` (`include_str!`), no I/O, no line splitting. Only the search.

| Strategy | Pattern | Matches | Time | Throughput |
| :--- | :--- | ---: | ---: | ---: |
| `Exact` | `zzzzzzzz` | 0 | 0.26 ms | 22.0 GB/s |
| `Exact` | `the` | 48,263 | 6.35 ms | 0.89 GB/s |
| `CaseInsensitive` | `zzzzzzzz` | 0 | 6.88 ms | 0.82 GB/s |
| `CaseInsensitive` | `the` | 56,854 | 11.20 ms | 0.50 GB/s |

What I see:

- `Exact` on a miss is crazy fast: one `str::find` on the whole text. The std search is very optimized.
- The file (5.64 MB) fits in L3 cache (24 MiB), so 22.0 GB/s is maybe the cache speed, not RAM speed.
- `CaseInsensitive` on a miss: 6.88 ms vs 0.26 ms. My loop tests every position byte by byte with `eq_ignore_ascii_case`. Idea from ripgrep (`memchr2`): first look for `z` or `Z`, and only compare the full pattern there.
- Hit vs miss for `CaseInsensitive`: 11.20 - 6.88 = ~4.3 ms to produce 56,854 matches (stop the loop, build `Match`, move the cursor, give it to the consumer).

### Printer (`benches/printer_bench.rs`)

The 39,634 lines that contain `the` (1.83 MB), written into a reused `Vec<u8>`.

> WARN: in this bench the lines come from `text.lines()`, without `\n`, so `suffix` writes a `\n` every time. In the real searcher the line keeps its `\n`. Small difference but it's not exactly the real path.

v1 = `write!` macro (commit `69c7600`), v2 = `write_all` with byte slices and a stack buffer for the line number (commit `11507e1`). Both measured with the same fixed bench.

| Flags | v1 (`write!`) | v2 (`write_all`) | Change |
| :--- | ---: | ---: | :--- |
| none | 523.1 µs | 277.7 µs | -46.9%, 1.9x faster |
| `-c` | 2.52 ms | 992.2 µs | -60.6%, 2.5x faster |
| `-n` | 1.27 ms | 475.7 µs | -62.5%, 2.7x faster |
| `-n -c` | 3.20 ms | 1.20 ms | -62.7%, 2.7x faster |

> WARN: my first printer table was wrong because of `io::sink()` (see the WARN above). With sink, the "none" case took 31 µs for 39,634 lines, 0.8 ns per line: only the loop, the printing was deleted. On `-n`, v2 even looked slower than v1. The real numbers are 3x to 9x higher, but now the comparison is honest.

What I learned: `write!` goes through `core::fmt`, even for a simple `{}` with a `&str`, and `write_all` with `&[u8]` skips it. The win is everywhere: 1.9x without flags, 2.5x with color (4 small writes per match: text before, green, match, reset) and 2.7x with line numbers (my stack buffer instead of formatting the `usize`).

### Searcher (`benches/searcher_bench.rs`)

The full loop: `BufReader` on the text in memory → `read_line` → matcher → printer → a reused `Vec<u8>`.

| Scenario | Pattern | Flags | Matching lines | Time | Throughput | Per line |
| :--- | :--- | :---: | ---: | ---: | ---: | ---: |
| `pure_miss_scan` | `zzzzzzzz` | none | 0 | 11.05 ms | 486.5 MiB/s | 56 ns |
| `realistic_sparse_hit` | `Hamlet` | `-n -c` | 105 | 13.63 ms | 394.6 MiB/s | 69 ns |
| `dense_stress_hit` | `the` | `-i -n -c` | 45,588 | 21.56 ms | 249.4 MiB/s | 110 ns |

What I see:

- Matcher alone on the full text (miss): 0.26 ms. Searcher with the same miss: 11 ms. So ~10.8 ms is a cost **per line**: 196k times `read_line` (find `\n`, copy into my `String`, check UTF-8) and 196k small `str::find` calls.
- My guess: most of it is `read_line` and the UTF-8 check but I didn't profile yet. I will test `perf` / flamegraph.
- The two buffers (`line` and `matches_collection`) are created once and reused with `.clear()`, so after the first lines there is no new allocation (I didn't count allocations yet to prove it).

---

## 3. What next

From this, the order I want to follow (details in `TODO.md`):

1. Profile (`perf` / flamegraph) to confirm where the 11 ms go.
2. Work on bytes (`read_until` / `&[u8]`) instead of `String`: no UTF-8 check on lines that don't match. It also fixes the error on non-UTF-8 files.
3. Search the pattern in big blocks first, and find the line only around a match (what grep and ripgrep do).
4. The `memchr2` idea for `-i`, with std only.
