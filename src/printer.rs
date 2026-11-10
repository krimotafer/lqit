use crate::matcher::Match;
use crate::parser::Config;
use std::io::{Result, Write};

// TODO: Make colors configurable

// actually i don't handle color in config i fix that here until i decide how i will handle that
const GREEN: &[u8] = b"\x1b[32m";
const RESET: &[u8] = b"\x1b[0m";

// utile function to convert line_number: usize to ASCII
// we use 20 octs slice - a stadard conversion size
fn usize_to_ascii(mut line_number: usize, buf: &mut [u8; 21]) -> &[u8] {
    // we use a reverse cursor in the slice because the conversion go right to left
    let mut reverse_cursor = buf.len() - 1;

    if line_number == 0 {
        reverse_cursor -= 1;
        buf[reverse_cursor] = b'0';
    } else {
        while line_number > 0 {
            reverse_cursor -= 1;
            buf[reverse_cursor] = b'0' + (line_number % 10) as u8;
            line_number /= 10;
        }
    }

    &buf[reverse_cursor..]
}

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
            // we use a buffer of 20 octs to get the standard max size for a usize +1 for the colone
            // ':'
            let mut buf: [u8; 21] = [0u8; 21];
            // add the colonne in the end ":"
            buf[buf.len() - 1] = b':';

            let ascii_line_number = usize_to_ascii(line_number, &mut buf);

            writer.write_all(ascii_line_number)?;
        }

        Ok(())
    }

    fn body<W: Write>(&self, writer: &mut W, line: &str, matches: &[Match]) -> Result<()> {
        let mut cursor: usize = 0;
        let line_bytes = line.as_bytes();

        // i choose to decouple the logic to not use new variable that force writer to write the
        // color or the an empty str "" (maybe some performance gain with that approach)
        if self.color {
            for m in matches {
                writer.write_all(&line_bytes[cursor..m.start])?;
                writer.write_all(GREEN)?;
                writer.write_all(&line_bytes[m.start..m.end])?;
                writer.write_all(RESET)?;

                cursor = m.end;
            }

            if cursor < line_bytes.len() {
                writer.write_all(&line_bytes[cursor..])?;
            }
        } else {
            writer.write_all(line_bytes)?;
        }

        Ok(())
    }

    fn suffix<W: Write>(&self, writer: &mut W, line: &str) -> Result<()> {
        if !line.ends_with('\n') {
            writer.write_all(b"\n")?;
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
        let target_print = "3:Hello from the test, you are in the TeSt to tEsT the printer\n";

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
            "3:Hello from the \x1b[32mtest\x1b[0m, you are in the TeSt to tEsT the printer\n";

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
