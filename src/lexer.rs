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

        let token = match arg.as_str() {
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
        };

        token
    }
}

/*
* 1. test_long_flag (Les options longues)
   Ce qu'on teste : Passer ["--help"].
   Ce qu'on attend : Obtenir Token::Long("help").
* 2. test_short_flags_combined (Le test STAR de ta machine à états !)
   Ce qu'on teste : Passer ["-in"].
   Ce qu'on attend :
   Premier .next() ➔ Token::Short('i')
   Deuxième .next() ➔ Token::Short('n')
   Troisième .next() ➔ None
* 3. test_values (Les arguments normaux)
   Ce qu'on teste : Passer ["needle", "file.txt"].
   Ce qu'on attend : Obtenir Token::Value("needle") puis Token::Value("file.txt").
* 4. test_edge_cases_dashes (Les cas limites du tiret)
   Ce qu'on teste : Passer ["-", "--"].
   Ce qu'on attend : Vérifier que ce sont bien des Token::Value et que le Lexer ne s'arrête pas par erreur.
* 5. test_empty_args (L'entrée vide)
   Ce qu'on teste : Passer une tranche vide &[].
   Ce qu'on attend : Le premier appel à .next() renvoie directement None.
*/

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
