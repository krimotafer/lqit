use crate::matcher::Matcher;
use crate::parser::Config;
use crate::printer::Printer;
use std::fs::File;
use std::io::{BufRead, BufReader, Result, Write};

// WARN:Here we use a fixed buffer to read each lines of a file but in production we can use 'memmap2' to
// let the OS put the file in memory and get bytes directly plus using. The String allocation of
// this case the String buffer allocation is minimal and i don't think that have a big impact in our
// mini implementation. But it's actually more reliable to use "memchar2" and "memmap2" to have full
// bytes handling application instead of string manipulations.
//
// WARN: Another question is about why we don't use the matcher iterator in the printer instead of
// using a collection. Ok this is my personal question in my learning path so i put it here
// and first i thought it's just overengineered if we refactor the printer to accept a generic Iterator
// instead of just using a small colleciton of matches, my thought was correct but i learn that it's a
// standard tradeoff! and if we want we can use 'peackable' iterator to check the future without
// consume the item of the iterator, it's really good to now!

//Facade pattern: Searcher handle matcher and printer
#[derive(Debug)]
pub struct Searcher<'a> {
    pub matcher: Matcher<'a>,
    pub printer: Printer,
}

impl<'a> Searcher<'a> {
    pub fn from_config(config: &'a Config) -> Self {
        let matcher = Matcher::from_config(config);
        let printer = Printer::from_config(config);

        Self { matcher, printer }
    }

    pub fn search<R: BufRead, W: Write>(&self, reader: &mut R, writer: &mut W) -> Result<()> {
        let mut line: String = String::new();
        // reusable buffer
        let mut matches_collection = Vec::new();
        let mut i: usize = 0;

        while reader.read_line(&mut line)? > 0 {
            i += 1;
            let matches = self.matcher.find_iter(&line);
            matches_collection.extend(matches);

            if !matches_collection.is_empty() {
                self.printer
                    .print_line(writer, &line, &matches_collection, i)?;
            }

            line.clear();
            matches_collection.clear();
        }

        Ok(())
    }

    pub fn search_file<W: Write>(&self, file_path: &str, writer: &mut W) -> Result<()> {
        let f = File::open(file_path)?;
        let mut reader = BufReader::new(f);
        self.search(&mut reader, writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_only_matching_lines() {
        let target = "first line\nsecond line with test\nthird line\nanother test here\n";
        let expected = "second line with test\nanother test here\n";

        let config = Config {
            pattern: "test",
            file_path: "",
            ignore_case: false,
            line_numbers: false,
            color: false,
        };

        let searcher = Searcher::from_config(&config);
        let mut reader = target.as_bytes();
        let mut writer = Vec::new();

        searcher.search(&mut reader, &mut writer).unwrap();

        assert_eq!(writer, expected.as_bytes());
    }

    #[test]
    fn line_numbers_track_skipped_lines() {
        let target = "first line\nsecond line with test\nthird line\nanother test here\n";
        let expected = "2:second line with test\n4:another test here\n";

        let config = Config {
            pattern: "test",
            file_path: "",
            ignore_case: false,
            line_numbers: true,
            color: false,
        };

        let searcher = Searcher::from_config(&config);
        let mut reader = target.as_bytes();
        let mut writer = Vec::new();

        searcher.search(&mut reader, &mut writer).unwrap();

        assert_eq!(writer, expected.as_bytes());
    }

    #[test]
    fn test_no_matches_produces_empty_output() {
        let target = "hello world\nrust is great\n";
        let expected = "";

        let config = Config {
            pattern: "test",
            file_path: "",
            ignore_case: false,
            line_numbers: true,
            color: true,
        };

        let searcher = Searcher::from_config(&config);
        let mut reader = target.as_bytes();
        let mut writer = Vec::new();

        searcher.search(&mut reader, &mut writer).unwrap();

        assert_eq!(writer, expected.as_bytes());
    }

    #[test]
    fn test_search_with_all_flags() {
        let target = "first line\nsecond line with TEST\nthird line\nanother test here\n";
        let expected =
            "2:second line with \x1b[32mTEST\x1b[0m\n4:another \x1b[32mtest\x1b[0m here\n";

        let config = Config {
            pattern: "test",
            file_path: "",
            ignore_case: true,
            line_numbers: true,
            color: true,
        };

        let searcher = Searcher::from_config(&config);
        let mut reader = target.as_bytes();
        let mut writer = Vec::new();

        searcher.search(&mut reader, &mut writer).unwrap();

        assert_eq!(writer, expected.as_bytes());
    }
}
