use std::slice::Iter;
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Short(char),
    Long(&'a str),
    Value(&'a str),
}

#[derive(Debug)]
pub struct Lexer<'a> {
    args: Iter<'a, String>,
    short_args: Option<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(args: &'a [String]) -> Self {
        Self {
            args: args.iter(),
            short_args: None,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // check first if there is no short args
        if let Some(chars) = &mut self.short_args {
            if let Some(c) = chars.next() {
                return Some(Token::Short(c));
            } else {
                self.short_args = None;
            }
        }

        let arg = self.args.next()?;

        match arg.as_str() {
            "-" => Some(Token::Value(arg)),
            "--" => Some(Token::Value(arg)),
            arg if arg.starts_with("--") => Some(Token::Long(&arg[2..])),
            arg if arg.starts_with('-') => {
                let mut chars = arg[1..].chars();
                let first_char = chars.next()?;
                self.short_args = Some(chars);
                Some(Token::Short(first_char))
            }
            _ => Some(Token::Value(arg)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_flag() {
        let flags = ["--help".to_string(), "--foo".to_string()];
        let mut lexer = Lexer::new(&flags);

        assert_eq!(Some(Token::Long("help")), lexer.next());
        assert_eq!(Some(Token::Long("foo")), lexer.next());
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_short_flags_combined() {
        let flags = ["-cnh".to_string()];
        let mut lexer = Lexer::new(&flags);

        assert_eq!(Some(Token::Short('c')), lexer.next());
        assert_eq!(Some(Token::Short('n')), lexer.next());
        assert_eq!(Some(Token::Short('h')), lexer.next());
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_values() {
        let flags = ["pattern".to_string(), "file.txt".to_string()];
        let mut lexer = Lexer::new(&flags);

        assert_eq!(Some(Token::Value("pattern")), lexer.next());
        assert_eq!(Some(Token::Value("file.txt")), lexer.next());
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_edge_cases_dashes() {
        let flags = ["-".to_string(), "--".to_string()];
        let mut lexer = Lexer::new(&flags);

        assert_eq!(Some(Token::Value("-")), lexer.next());
        assert_eq!(Some(Token::Value("--")), lexer.next());
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_empty_args() {
        let mut lexer = Lexer::new(&[]);

        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_mixed_flags() {
        let flags = [
            "-cnh".to_string(),
            "pattern".to_string(),
            "--foo".to_string(),
            "file.txt".to_string(),
            "-i".to_string(),
            "--".to_string(),
        ];
        let mut lexer = Lexer::new(&flags);

        assert_eq!(Some(Token::Short('c')), lexer.next());
        assert_eq!(Some(Token::Short('n')), lexer.next());
        assert_eq!(Some(Token::Short('h')), lexer.next());

        assert_eq!(Some(Token::Value("pattern")), lexer.next());

        assert_eq!(Some(Token::Long("foo")), lexer.next());

        assert_eq!(Some(Token::Value("file.txt")), lexer.next());

        assert_eq!(Some(Token::Short('i')), lexer.next());

        assert_eq!(Some(Token::Value("--")), lexer.next());

        assert_eq!(None, lexer.next());
    }
}
