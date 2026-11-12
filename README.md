# minigrep

A small `grep` in Rust, only with the standard library. I build it to learn.

> [!IMPORTANT]
> **This is a learning project.** I use it to learn Rust and software engineering: design patterns, trade-offs, zero-copy, measuring performance. It's not a competitor to [ripgrep](https://github.com/BurntSushi/ripgrep) or [GNU grep](https://www.gnu.org/software/grep/), and that's not the goal. It's slower than both (numbers below), and that's OK.

> [!NOTE]
> **About AI.** The Rust code in `src/` is mine, no AI rewrite. I used AI for the boring parts: the searcher Criterion bench, the hyperfine script, and turning my notes, code comments and commits into `README.md`, `BENCHMARKS.md` and `TODO.md` (Claude). The ideas and the mistakes in the code are mine.

## Why

The Rust book has a chapter where you build a mini grep, and it recommends reading the [ripgrep](https://github.com/BurntSushi/ripgrep) code from BurntSushi. I did the book version first in "blind mode", without looking at the book's code. Then I wanted to go deeper: what is behind the scene, where are the limits, which trade-offs we have to make. And of course train my brain on design patterns and zero-copy Rust.

My rules:

- **No external crate, only the std.** I want to understand the fundamentals and the limits of the std before using `memchr` or `memmap2`. (Criterion is only a dev-dependency for the benches.)
- **Zero-copy when I can**: borrow (`&str`) instead of clone.
- **Measure before I believe**: Criterion for each component, hyperfine for the full CLI.
- **Write my thoughts down**: in code comments (look for `WARN:` and `TODO:`) and in commit messages. This README is the short version.

## Usage

```
minigrep [OPTION]... PATTERN FILE

  -i, --ignore-case     ignore case distinctions in patterns and data
  -n, --line-numbers    print line number with output lines
  -c, --color           color the matches
  -h, --help            print the help
```

```bash
cargo run --release -- -in hamlet data/pg100.txt
```

Short flags can be combined (`-inc`). Careful: `-c` is color here, not count like in grep.

## How it works

```
args ──► Lexer ──► Parser ──► Config ──► Searcher ──► stdout
                                          ├─ Matcher (find the matches in a line)
                                          └─ Printer (write the line)
```

| Module | What it does | What I practiced |
| :--- | :--- | :--- |
| `lexer.rs` | Args → `Token` (`Short`, `Long`, `Value`). `-inc` gives 3 tokens. | Iterator by hand, lifetimes: tokens borrow from the args, no copy. |
| `parser.rs` | Tokens → `Config`, or a typed `ParserError`. | Zero-copy `Config<'a>`, errors that borrow too, `ok_or` and `?`. |
| `matcher.rs` | Finds the matches in a line: `Empty`, `Exact`, `CaseInsensitive`. | Strategy pattern with an enum, factory, lazy Iterator. |
| `printer.rs` | Writes the line: line number, colors, final `\n`. | Generic `W: Write` (dependency injection, easy to test with `Vec<u8>`), composed method, `write_all` on bytes. |
| `searcher.rs` | Reads line by line and connects matcher and printer. | Facade, generic `BufRead`, buffers reused with `.clear()`. |
| `main.rs` | Wires everything. | One `stdout().lock()`, `BufWriter`, `BrokenPipe` = silent exit 0. |

## What I learned

The short version of my comments and commits:

- **The lexer took me 3 tries.** First a `Vec` version (not even working), then an `Iterator` that keeps a `Chars` sub-iterator to split `-inc`, then I moved everything to a lib and the lexer borrows the slice of args. No token is allocated.
- **`eq_ignore_ascii_case` is ASCII only.** `café` doesn't match `CAFÉ` because `é` is `0xC3 0xA9` and `É` is `0xC3 0x89`. The std avoids shipping the big Unicode table for that, as I understand it.
- **Bytes avoid UTF-8 bugs.** In `CaseInsensitive` I compare bytes, so I never cut a `&str` in the middle of a char.
- **The empty pattern matches everything.** Zero-width match at every position, the ε case of formal language theory.
- **`write!` vs `write_all`.** `write!` goes through `core::fmt`. With `write_all` on bytes my printer is 1.8x faster with colors.
- **Slice vs iterator.** The printer takes a small slice of matches instead of the matcher's iterator. Simpler, and it's a standard trade-off. If one day I need to look ahead without consuming, `peekable` exists.
- **Kernel time is not my problem.** One lock on stdout + `BufWriter` → ~1 ms of system time, same as ripgrep.
- **My bottleneck is not the search.** It's reading line by line into a `String` (UTF-8 check on each of the 196k lines). That's why grep and ripgrep work on raw bytes.
- **Benchmarking is tricky.** My first comparison with grep was wrong: when the output goes to `/dev/null`, GNU grep stops at the first match. Details in [BENCHMARKS.md](BENCHMARKS.md).

## Benchmarks (short)

Shakespeare complete works (5.64 MB, 196k lines), Intel Core Ultra 7 155H, full CLI with hyperfine:

| Search | minigrep | GNU grep | ripgrep |
| :--- | ---: | ---: | ---: |
| `Hamlet` | 15.8 ms | 5.5 ms | 2.7 ms |
| `the` (39k lines) | 19.3 ms | 11.3 ms | 7.3 ms |
| `-in Hamlet` | 19.9 ms | 10.8 ms | 4.1 ms |
| no match | 14.9 ms | 1.7 ms | 2.8 ms |

Yes it's slower, see the disclaimer :) All the numbers, Criterion per component, and what I understand from them: [BENCHMARKS.md](BENCHMARKS.md).

## Known limits

- One file only, no stdin.
- Can't search a pattern that starts with `-`.
- A non-UTF-8 file stops the search with an error, this why `memchar` is matter to play directly with bytes and avoid UTF-8 checking.
- `-i` is ASCII only.
- Exit code is 0 even when nothing matches (grep returns 1).

All of them are in [TODO.md](TODO.md).

## Build, test, bench

```bash
cargo build --release
cargo test
cargo bench
./scripts/bench_cli.sh   # needs hyperfine, ripgrep is optional
```

## About me

I'm doing a PhD in another field, Rust is my side quest, so this project grows slowly (PhD life, don't judge please :) ).
