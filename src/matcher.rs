use crate::parser::Config;

// WARN:Current bottleneck is the case-insensitive search algorithm:
// We spend about 4 ms updating the cursor and yielding matches.
// ripgrep use memchr2 to get the actual performance.
// Check BENCHMARKS.md for more details.

// WARN: By using ascii comparaison with 'eq_ignore_ascii_case' in CaseInsensitive option
// i don't take into non-ASCII characters for example :
// 'café' pattern dooesn't match 'CAFÉ' due to the multi bytes representation so
// 'é' (0xC3 0xA9) != 'É' (0xC3 0x89).
// Rust std avoid the huge unicode table to avoid massive lookup.
// As i undersatand it !

// TODO: Improve case-insensitive algorithm using memchr2.

#[derive(Debug, PartialEq, Eq)]
pub struct Match {
    pub start: usize,
    pub end: usize,
}

// strategy pattern: List each search algorithm
#[derive(Debug)]
pub enum Matcher<'a> {
    // The empty special case should return all the file to try to respect the formal langage
    // thoery and 𝜖 case (zero-width matches)
    Empty,
    Exact(&'a str),
    // we can use in this case =>  eq_ignore_ascii_case == go check str in the doc
    CaseInsensitive(&'a str),
    // TODO: add regex
}

// factory pattern: Instantiates the strategy from config
impl<'a> Matcher<'a> {
    pub fn from_config(config: &'a Config) -> Self {
        if config.pattern.is_empty() {
            Self::Empty
        } else if config.ignore_case {
            Self::CaseInsensitive(config.pattern)
        } else {
            Self::Exact(config.pattern)
        }
    }

    pub fn find_iter<'b>(&'a self, line: &'b str) -> Matches<'a, 'b> {
        Matches::new(self, line)
    }
}

#[derive(Debug)]
pub struct Matches<'a, 'b> {
    pub matcher: &'a Matcher<'a>,
    pub line: &'b str,
    pub cursor: usize,
}

impl<'a, 'b> Matches<'a, 'b> {
    fn new(matcher: &'a Matcher<'a>, line: &'b str) -> Self {
        Self {
            matcher,
            line,
            cursor: 0,
        }
    }
}

// Iterator pattern: Lazy and zero-copy
impl<'a, 'b> Iterator for Matches<'a, 'b> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        match self.matcher {
            Matcher::Empty => {
                if self.cursor > self.line.len() {
                    return None;
                }

                let start = self.cursor;
                let end = self.cursor;

                self.cursor += 1;

                Some(Match { start, end })
            }
            Matcher::Exact(pattern) => {
                let slice = &self.line[self.cursor..];

                slice.find(pattern).map(|pos| {
                    let start = self.cursor + pos;
                    let end = start + pattern.len();
                    self.cursor = end;
                    Match { start, end }
                })
            }
            Matcher::CaseInsensitive(pattern) => {
                // use bytes instead of str because of utf8 bugs
                // if we have special character in 2 bytes we lose the comparaisan
                let line_bytes = self.line.as_bytes();
                let pattern_bytes = pattern.as_bytes();

                let max_cursor_start = line_bytes.len().checked_sub(pattern_bytes.len())?;

                if self.cursor > line_bytes.len() {
                    return None;
                }

                for i in self.cursor..=max_cursor_start {
                    let slice_end = i + pattern_bytes.len();

                    let slice = &line_bytes[i..slice_end];

                    // does not match non-ASCII cases like 'é' vs 'É'
                    if slice.eq_ignore_ascii_case(pattern_bytes) {
                        self.cursor = slice_end;
                        return Some(Match {
                            start: i,
                            end: self.cursor,
                        });
                    }
                }

                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let target = "Hello from the test, you are in the TeSt to tEsT the matcher";

        let config = Config {
            pattern: "test",
            file_path: "",
            ignore_case: false,
            line_numbers: false,
            color: false,
        };

        let matcher = Matcher::from_config(&config);
        let results: Vec<_> = matcher.find_iter(target).collect();

        let first_result = Match { start: 15, end: 19 };

        assert_eq!(results, vec![first_result]);
    }

    #[test]
    fn test_case_insensitive_match() {
        let target = "Hello from the test, you are in the TeSt to tEsT the matcher";

        let config = Config {
            pattern: "test",
            file_path: "",
            ignore_case: true,
            line_numbers: false,
            color: false,
        };

        let matcher = Matcher::from_config(&config);
        let results: Vec<_> = matcher.find_iter(target).collect();

        let first_result = Match { start: 15, end: 19 };
        let second_result = Match { start: 36, end: 40 };
        let third_result = Match { start: 44, end: 48 };

        assert_eq!(results, vec![first_result, second_result, third_result]);
    }

    #[test]
    fn test_line_shorter_than_pattern() {
        let target = "hi";
        let config = Config {
            pattern: "hello_world",
            file_path: "",
            ignore_case: true,
            line_numbers: false,
            color: false,
        };

        let matcher = Matcher::from_config(&config);

        assert_eq!(matcher.find_iter(target).next(), None);
    }

    #[test]
    fn test_empty_pattern() {
        let target = "Hello from the test, you are in the TeSt to tEsT the matcher";

        let config = Config {
            pattern: "",
            file_path: "",
            ignore_case: true,
            line_numbers: false,
            color: false,
        };

        let matcher = Matcher::from_config(&config);
        let results: Vec<_> = matcher.find_iter(target).collect();

        let mut true_results: Vec<_> = Vec::new();
        for i in 0..=target.len() {
            true_results.push(Match { start: i, end: i });
        }

        assert_eq!(results, true_results);
    }
    #[test]
    fn test_empty_pattern_on_empty_line() {
        let target = "";
        let config = Config {
            pattern: "",
            file_path: "",
            ignore_case: false,
            line_numbers: false,
            color: false,
        };
        let matcher = Matcher::from_config(&config);
        let results: Vec<_> = matcher.find_iter(target).collect();

        assert_eq!(results, vec![Match { start: 0, end: 0 }]);
    }
}
