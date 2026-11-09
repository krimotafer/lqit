use crate::matcher::Match;
use crate::parser::Config;
use std::io::{Result, Write};

// TODO: Avoid using write! macro, use write_all instead: As i see the write! macro call core::fmt and
// there is some adaptater to handle io::Result (as my little understanding) so instead to avoid
// overhead. i think i will rewrite the printer with write_all and like ripgrep use [u8] instead of
// str. We need to take into consideration line number "usize" -> [u8] zero copy !
// TODO: Make colors configurable

// actually i don't handle color in config i fix that here until i decide how i will handle that
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

#[derive(Debug, Eq, PartialEq)]
pub struct Printer {
    line_numbers: bool,
    color: bool,
}

impl Printer {
    // static factory pattern
    pub fn from_config(config: &Config) -> Self {
        Self {
            line_numbers: config.line_numbers,
            color: config.color,
        }
    }

    // use sink pattern with "Write" (dependecy injection) to write to any destination (stdout, file ...)
    pub fn print_line<W: Write>(
        &self,
        writer: &mut W,
        line: &str,
        matches: &[Match],
        line_number: usize,
    ) -> Result<()> {
        self.prefix(writer, line_number)?;
        self.body(writer, line, matches)?;
        self.suffix(writer, line)?;
        Ok(())
    }

    // As i know this is a pipeline pattern: prefix -> body -> suffix

    fn prefix<W: Write>(&self, writer: &mut W, line_number: usize) -> Result<()> {
        if self.line_numbers {
            write!(writer, "{}: ", line_number)?;
        }

        Ok(())
    }

    fn body<W: Write>(&self, writer: &mut W, line: &str, matches: &[Match]) -> Result<()> {
        let mut cursor: usize = 0;

        // i choose to decouple the logic to not use new variable that force writer to write the
        // color or the an empty str "" (maybe some performance gain with that approach)
        if self.color {
            for m in matches {
                write!(
                    writer,
                    "{}{GREEN}{}{RESET}",
                    &line[cursor..m.start],
                    &line[m.start..m.end]
                )?;
                cursor = m.end;
            }

            if cursor < line.len() {
                write!(writer, "{}", &line[cursor..])?;
            }
        } else {
            write!(writer, "{}", &line)?;
        }

        Ok(())
    }

    fn suffix<W: Write>(&self, writer: &mut W, line: &str) -> Result<()> {
        if !line.ends_with('\n') {
            writeln!(writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::Matcher;

    #[test]
    fn test_without_args() {
        let target = "Hello from the test, you are in the TeSt to tEsT the printer\n";

        let config = Config {
            pattern: "test",
            file_path: "",
            ignore_case: false,
            line_numbers: false,
            color: false,
        };

        let matcher = Matcher::from_config(&config);
        let results: Vec<_> = matcher.find_iter(target).collect();

        // printer
        let printer = Printer::from_config(&config);
        let mut vec_writer: Vec<u8> = Vec::new();

        printer
            .print_line(&mut vec_writer, target, &results, 3)
            .unwrap();

        assert_eq!(target.as_bytes(), vec_writer);
    }

    #[test]
    fn test_line_numbers() {
        let target = "Hello from the test, you are in the TeSt to tEsT the printer\n";
        let target_print = "3: Hello from the test, you are in the TeSt to tEsT the printer\n";

        let config = Config {
            pattern: "test",
            file_path: "",
            ignore_case: false,
            line_numbers: true,
            color: false,
        };

        let matcher = Matcher::from_config(&config);
        let results: Vec<_> = matcher.find_iter(target).collect();

        // printer
        let printer = Printer::from_config(&config);
        let mut vec_writer: Vec<u8> = Vec::new();

        printer
            .print_line(&mut vec_writer, target, &results, 3)
            .unwrap();

        assert_eq!(target_print.as_bytes(), vec_writer);
    }

    #[test]
    fn test_color() {
        let target = "Hello from the test, you are in the TeSt to tEsT the printer\n";
        let target_print =
            "Hello from the \x1b[32mtest\x1b[0m, you are in the TeSt to tEsT the printer\n";

        let config = Config {
            pattern: "test",
            file_path: "",
            ignore_case: false,
            line_numbers: false,
            color: true,
        };

        let matcher = Matcher::from_config(&config);
        let results: Vec<_> = matcher.find_iter(target).collect();

        // printer
        let printer = Printer::from_config(&config);
        let mut vec_writer: Vec<u8> = Vec::new();

        printer
            .print_line(&mut vec_writer, target, &results, 3)
            .unwrap();

        assert_eq!(target_print.as_bytes(), vec_writer);
    }

    #[test]
    fn test_all_args() {
        let target = "Hello from the test, you are in the TeSt to tEsT the printer\n";
        let target_print =
            "3: Hello from the \x1b[32mtest\x1b[0m, you are in the TeSt to tEsT the printer\n";

        let config = Config {
            pattern: "test",
            file_path: "",
            ignore_case: false,
            line_numbers: true,
            color: true,
        };

        let matcher = Matcher::from_config(&config);
        let results: Vec<_> = matcher.find_iter(target).collect();

        // printer
        let printer = Printer::from_config(&config);
        let mut vec_writer: Vec<u8> = Vec::new();

        printer
            .print_line(&mut vec_writer, target, &results, 3)
            .unwrap();

        assert_eq!(target_print.as_bytes(), vec_writer);
    }
}
