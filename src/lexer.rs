use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Tempo,
    Scale,
    Section,
    Chords,
    Bass,
    Melody,
    Drums,
    // Symbols
    LBrace,
    RBrace,
    Pipe,
    // Values
    Number(u32),
    Ident(String),
    // Control
    Newline,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Ident(s) => write!(f, "{}", s),
            other => write!(f, "{:?}", other),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer { input: input.chars().collect(), pos: 0 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok == Token::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        self.pos += 1;
        ch
    }

    fn skip_whitespace_no_newline(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_no_newline();

        match self.peek() {
            None => Ok(Token::Eof),
            Some('\n') => {
                self.advance();
                // collapse multiple newlines
                while self.peek() == Some('\n') || self.peek() == Some('\r') {
                    self.advance();
                }
                Ok(Token::Newline)
            }
            Some('/') => {
                // comment: skip to end of line
                while self.peek().is_some() && self.peek() != Some('\n') {
                    self.advance();
                }
                self.next_token()
            }
            Some('{') => { self.advance(); Ok(Token::LBrace) }
            Some('}') => { self.advance(); Ok(Token::RBrace) }
            Some('|') => { self.advance(); Ok(Token::Pipe) }
            Some(ch) if ch.is_ascii_digit() => {
                let mut s = String::new();
                while let Some(d) = self.peek() {
                    if d.is_ascii_digit() { s.push(d); self.advance(); } else { break; }
                }
                Ok(Token::Number(s.parse().unwrap()))
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let mut s = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '#' {
                        s.push(c); self.advance();
                    } else {
                        break;
                    }
                }
                Ok(match s.as_str() {
                    "tempo"   => Token::Tempo,
                    "scale"   => Token::Scale,
                    "section" => Token::Section,
                    "chords"  => Token::Chords,
                    "bass"    => Token::Bass,
                    "melody"  => Token::Melody,
                    "drums"   => Token::Drums,
                    _         => Token::Ident(s),
                })
            }
            Some(ch) => Err(format!("unexpected char: {:?}", ch)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(src: &str) -> Vec<Token> {
        Lexer::new(src).tokenize().unwrap()
    }

    fn lex_no_eof(src: &str) -> Vec<Token> {
        let mut t = lex(src);
        t.retain(|tok| tok != &Token::Eof && tok != &Token::Newline);
        t
    }

    #[test]
    fn keywords_recognized() {
        let tokens = lex_no_eof("tempo scale section chords bass melody drums");
        assert_eq!(tokens, vec![
            Token::Tempo, Token::Scale, Token::Section,
            Token::Chords, Token::Bass, Token::Melody, Token::Drums,
        ]);
    }

    #[test]
    fn number_parsed() {
        let tokens = lex_no_eof("120");
        assert_eq!(tokens, vec![Token::Number(120)]);
    }

    #[test]
    fn identifier_parsed() {
        let tokens = lex_no_eof("C_major");
        assert_eq!(tokens, vec![Token::Ident("C_major".into())]);
    }

    #[test]
    fn note_with_sharp_parsed_as_ident() {
        let tokens = lex_no_eof("F#4");
        assert_eq!(tokens, vec![Token::Ident("F#4".into())]);
    }

    #[test]
    fn symbols_parsed() {
        let tokens = lex_no_eof("{ } |");
        assert_eq!(tokens, vec![Token::LBrace, Token::RBrace, Token::Pipe]);
    }

    #[test]
    fn comment_skipped() {
        let tokens = lex_no_eof("tempo // this is a comment\n120");
        assert_eq!(tokens, vec![Token::Tempo, Token::Number(120)]);
    }

    #[test]
    fn multiple_newlines_collapsed() {
        let tokens = lex("a\n\n\nb");
        let newlines = tokens.iter().filter(|t| *t == &Token::Newline).count();
        assert_eq!(newlines, 1);
    }

    #[test]
    fn unknown_char_returns_error() {
        let result = Lexer::new("@").tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn empty_input_returns_eof() {
        let tokens = lex("");
        assert_eq!(tokens, vec![Token::Eof]);
    }

    #[test]
    fn full_header_tokenized() {
        let tokens = lex_no_eof("tempo 120\nscale C_major");
        assert_eq!(tokens, vec![
            Token::Tempo, Token::Number(120),
            Token::Scale, Token::Ident("C_major".into()),
        ]);
    }
}
