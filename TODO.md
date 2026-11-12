# TODO

My backlog. What is done is in the commits and the code comments.

## Lexer (`src/lexer.rs`)

- [ ] `--` must stop the options: everything after it is a value. Today `minigrep -- -v file` fails with `unknown option '-v'`, so I can't search a pattern that starts with `-`. My test `test_edge_cases_dashes` expects `--` as a value, I need to change it too.
- [ ] `env::args_os()` instead of `env::args()`: `args()` panics if an argument is not valid UTF-8 (a file path on Linux can be anything).
- [ ] `--option=value` with `split_once('=')`.
- [ ] Attached short values (`-n5`, `-fpattern`).
- [ ] Maybe work on bytes (`b'-'`) instead of `chars()` for the flags.

## Parser (`src/parser.rs`)

- [ ] stdin: `file_path: Option<&str>`, and `-` means stdin (`cat file.txt | minigrep pattern`). `Searcher::search` is already generic on `BufRead`, so it should be easy.
- [ ] Tests for the `Display` of each `ParserError`.

## Matcher (`src/matcher.rs`)

- [ ] `Empty` moves the cursor 1 byte at a time. On non-ASCII text with `-c`, the color codes are written in the middle of a char (`é` = `0xC3 0xA9` is cut in 2). Move by char, not by byte.
- [ ] `-i` is a naive byte loop: try the `memchr2` idea (look for `h` or `H` first, then compare the rest), std only.
- [ ] Unicode case folding (`café` vs `CAFÉ`), known limit, see the WARN in `matcher.rs`.
- [ ] Regex, one day.

## Printer (`src/printer.rs`)

- [ ] Colors configurable.
- [ ] Tests for `usize_to_ascii` (`0` and `usize::MAX`).

## Searcher and main (`src/searcher.rs`, `src/main.rs`)

- [ ] Without `-c` I don't need all the matches, only to know if there is one (`.next().is_some()`). Today I collect all of them for nothing.
- [ ] A non-UTF-8 file stops everything: `read_line` returns `stream did not contain valid UTF-8`. Moving to bytes fixes this, and it's also my bottleneck (see `BENCHMARKS.md`).
- [ ] Call `writer.flush()` at the end: when `BufWriter` is dropped, a flush error is ignored.
- [ ] Different messages for read errors and write errors (today both are `minigrep: <file>: ...`).
- [ ] Several files.

## Performance (bigger steps)

- [ ] Profile with `perf` / flamegraph before I trust my guess about `read_line` and UTF-8.
- [ ] Bytes everywhere (`read_until` / `&[u8]`), UTF-8 check only when I print.
- [ ] Search in big blocks first, and find the line only around a match.
- [ ] Later, when I allow crates: `memchr`, `memmap2`, threads for many files.
