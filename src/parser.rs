use crate::lexer::{Lexer, Token};
use std::fmt;

const HELP: &str = "Usage: minigrep [OPTION]... PATTERN FILE
Search for PATTERN in FILE.
Example: minigrep -i 'hello world' menu.h main.c

Pattern selection and interpretation:
  -i, --ignore-case     ignore case distinctions in patterns and data
  -n, --line-numbers    print line number with output lines
  -c, --color           color the matches

";

#[derive(Debug, PartialEq)]
pub struct Config<'a> {
    pub pattern: &'a str,
    pub file_path: &'a str,
    pub ignore_case: bool,  // -i ignore-case
    pub line_numbers: bool, // -n line-numbers
    pub color: bool,        // -c
}

#[derive(Debug, PartialEq)]
pub enum ParserError<'a> {
    MissingPattern,
    MissingFilePath,
    TooManyArguments(&'a str),
    UnknownShortOption(char),
    UnknownLongOption(&'a str),
    HelpRequest,
}

impl<'a> Config<'a> {
    pub fn parse(lexer: Lexer<'a>) -> Result<Self, ParserError<'a>> {
        let mut pattern: Option<&str> = None;
        let mut file_path: Option<&str> = None;

        let mut ignore_case: bool = false;
        let mut line_numbers: bool = false;
        let mut color: bool = false;

        for token in lexer {
            match token {
                Token::Short('h') => return Err(ParserError::HelpRequest),
                Token::Short('i') => {
                    ignore_case = true;
                }
                Token::Short('n') => {
                    line_numbers = true;
                }
                Token::Short('c') => {
                    color = true;
                }

                Token::Short(short) => return Err(ParserError::UnknownShortOption(short)),

                Token::Long("help") => return Err(ParserError::HelpRequest),
                Token::Long("ignore-case") => {
                    ignore_case = true;
                }
                Token::Long("line-numbers") => {
                    line_numbers = true;
                }
                Token::Long("color") => {
                    color = true;
                }

                Token::Long(long) => return Err(ParserError::UnknownLongOption(long)),

                Token::Value(value) => match (pattern, file_path) {
                    (None, _) => pattern = Some(value),
                    (Some(_), None) => file_path = Some(value),
                    (Some(_), Some(_)) => return Err(ParserError::TooManyArguments(value)),
                },
            }
        }

        Ok(Self {
            pattern: pattern.ok_or(ParserError::MissingPattern)?,
            file_path: file_path.ok_or(ParserError::MissingFilePath)?,
            ignore_case,
            line_numbers,
            color,
        })
    }
}

impl<'a> fmt::Display for ParserError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::MissingPattern => write!(f, "missing 'pattern'\n{}", HELP),
            ParserError::MissingFilePath => write!(f, "missing 'file path'\n{}", HELP),
            ParserError::UnknownShortOption(short) => {
                write!(f, "unknown option '-{}'\n{}", short, HELP)
            }
            ParserError::UnknownLongOption(long) => {
                write!(f, "unknown option '--{}'\n{}", long, HELP)
            }
            ParserError::TooManyArguments(arg) => write!(f, "too many arguments {}\n{}", arg, HELP),
            ParserError::HelpRequest => write!(f, "{}", HELP),
        }
    }
}

impl<'a> std::error::Error for ParserError<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_input_short_options() {
        let flags = [
            "-inc".to_string(),
            "pattern".to_string(),
            "file.txt".to_string(),
        ];

        let lexer = Lexer::new(&flags);
        let config = Config::parse(lexer);

        assert_eq!(
            Ok(Config {
                pattern: "pattern",
                file_path: "file.txt",
                ignore_case: true,
                line_numbers: true,
                color: true,
            }),
            config
        );
    }

    #[test]
    fn test_clean_input_long_options() {
        let flags = [
            "pattern".to_string(),
            "file.txt".to_string(),
            "--ignore-case".to_string(),
            "--line-numbers".to_string(),
            "--color".to_string(),
        ];

        let lexer = Lexer::new(&flags);
        let config = Config::parse(lexer);

        assert_eq!(
            Ok(Config {
                pattern: "pattern",
                file_path: "file.txt",
                ignore_case: true,
                line_numbers: true,
                color: true,
            }),
            config
        );
    }

    #[test]
    fn test_short_help_request() {
        let flags = ["-h".to_string()];

        let lexer = Lexer::new(&flags);
        let config = Config::parse(lexer);

        assert_eq!(config, Err(ParserError::HelpRequest));
    }

    #[test]
    fn test_long_help_request() {
        let flags = ["--help".to_string()];

        let lexer = Lexer::new(&flags);
        let config = Config::parse(lexer);

        assert_eq!(config, Err(ParserError::HelpRequest));
    }

    #[test]
    fn test_help_request_with_other_flags() {
        let flags = ["-inh".to_string()];

        let lexer = Lexer::new(&flags);
        let config = Config::parse(lexer);

        assert_eq!(config, Err(ParserError::HelpRequest));
    }

    #[test]
    fn test_missing_pattern() {
        let flags = ["-in".to_string()];

        let lexer = Lexer::new(&flags);
        let config = Config::parse(lexer);

        assert_eq!(config, Err(ParserError::MissingPattern));
    }

    #[test]
    fn test_missing_file_path() {
        let flags = ["-in".to_string(), "pattern".to_string()];

        let lexer = Lexer::new(&flags);
        let config = Config::parse(lexer);

        assert_eq!(config, Err(ParserError::MissingFilePath));
    }

    #[test]
    fn test_uknown_option_short() {
        let flags = [
            "-inf".to_string(),
            "pattern".to_string(),
            "file.txt".to_string(),
        ];

        let lexer = Lexer::new(&flags);
        let config = Config::parse(lexer);

        assert_eq!(config, Err(ParserError::UnknownShortOption('f')));
    }

    #[test]
    fn test_uknown_option_long() {
        let flags = [
            "-in".to_string(),
            "pattern".to_string(),
            "file.txt".to_string(),
            "--foo".to_string(),
        ];

        let lexer = Lexer::new(&flags);
        let config = Config::parse(lexer);

        assert_eq!(config, Err(ParserError::UnknownLongOption("foo")));
    }

    #[test]
    fn test_too_many_args() {
        let flags = [
            "-in".to_string(),
            "pattern".to_string(),
            "file.txt".to_string(),
            "foo".to_string(),
        ];

        let lexer = Lexer::new(&flags);
        let config = Config::parse(lexer);

        assert_eq!(config, Err(ParserError::TooManyArguments("foo")));
    }

    #[test]
    fn test_mixed_args() {
        let flags = [
            "-in".to_string(),
            "pattern".to_string(),
            "file.txt".to_string(),
            "--ignore-case".to_string(),
            "--line-numbers".to_string(),
        ];

        let lexer = Lexer::new(&flags);
        let config = Config::parse(lexer);

        assert_eq!(
            Ok(Config {
                pattern: "pattern",
                file_path: "file.txt",
                ignore_case: true,
                line_numbers: true,
                color: false,
            }),
            config
        );
    }
}
