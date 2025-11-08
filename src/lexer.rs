use crate::{
    diagnostic::{Diagnostic, Level},
    span::Span,
    token::{Delim, HttpMethod, Keyword, Token, TokenKind},
};

pub fn lex<'input>(input: &'input str) -> Result<Vec<Token<'input>>, Diagnostic> {
    let mut lexer = Lexer::new(input);
    lexer.lex()
}

struct Lexer<'input> {
    input: &'input str,
    pos: usize,
}

impl<'input> Lexer<'input> {
    fn new(input: &'input str) -> Self {
        Self { input, pos: 0 }
    }

    fn lex(&mut self) -> Result<Vec<Token<'input>>, Diagnostic> {
        let mut tokens = vec![];
        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token<'input>>, Diagnostic> {
        self.skip_whitespace();
        let Some(first) = self.first() else {
            return Ok(None);
        };

        let start = self.pos;
        self.bump();

        let kind = match first {
            ':' => TokenKind::Colon,
            '{' => TokenKind::OpenDelim(Delim::Brace),
            '}' => TokenKind::CloseDelim(Delim::Brace),
            '"' => self.string(start)?,
            _ if first.is_ascii_digit() => self.integer(start),
            _ if first.is_alphabetic() || first == '_' => self.identifier(start),
            _ => {
                let span = Span::new(start, start);
                return Err(Diagnostic::error("Unrecognized character", span).label(
                    "I don't know what to do with this character",
                    span,
                    Level::Error,
                ));
            }
        };

        let span = Span::new(start, self.pos);
        Ok(Some(Token::new(kind, span)))
    }

    fn string(&mut self, start: usize) -> Result<TokenKind<'input>, Diagnostic> {
        while let Some(ch) = self.next() {
            if ch == '"' {
                return Ok(TokenKind::String(&self.input[start..self.pos]));
            }

            if ch == '\\' {
                self.bump();
            }
        }

        let span = Span::new(start, self.pos);
        Err(
            Diagnostic::error("Unterminated string literal", span).label(
                "I never found the closing quote for this string",
                span,
                Level::Error,
            ),
        )
    }

    fn integer(&mut self, start: usize) -> TokenKind<'input> {
        while let Some(ch) = self.first() {
            if !ch.is_ascii_digit() {
                break;
            }
            self.bump();
        }

        let text = &self.input[start..self.pos];
        TokenKind::Integer(text)
    }

    fn identifier(&mut self, start: usize) -> TokenKind<'input> {
        while let Some(ch) = self.first() {
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            self.bump();
        }

        let text = &self.input[start..self.pos];
        match text {
            "entry" => TokenKind::Keyword(Keyword::Entry),
            "GET" => TokenKind::HttpMethod(HttpMethod::Get),
            _ => TokenKind::Identifier(text),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.first() {
            if !ch.is_whitespace() {
                break;
            }
            self.bump();
        }
    }

    fn first(&mut self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn next(&mut self) -> Option<char> {
        if let Some(ch) = self.first() {
            self.pos += ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }

    fn bump(&mut self) {
        _ = self.next();
    }
}

#[cfg(test)]
mod test {
    use crate::{span::Span, token::Keyword};

    use super::*;

    #[test]
    fn lex_identifier_simple() {
        assert_token(
            "foo",
            Token::new(TokenKind::Identifier("foo"), Span::new(0, 3)),
        );
    }

    #[test]
    fn lex_identifier_with_underscore() {
        assert_token(
            "foo_bar",
            Token::new(TokenKind::Identifier("foo_bar"), Span::new(0, 7)),
        );
    }

    #[test]
    fn lex_identifier_with_leading_underscore() {
        assert_token(
            "_foobar",
            Token::new(TokenKind::Identifier("_foobar"), Span::new(0, 7)),
        );
    }

    #[test]
    fn lex_identifier_with_digits() {
        assert_token(
            "foo123",
            Token::new(TokenKind::Identifier("foo123"), Span::new(0, 6)),
        );
    }

    #[test]
    fn lex_identifier_get() {
        assert_token(
            "get",
            Token::new(TokenKind::Identifier("get"), Span::new(0, 3)),
        );
    }

    #[test]
    fn lex_http_method_get() {
        assert_token(
            "GET",
            Token::new(TokenKind::HttpMethod(HttpMethod::Get), Span::new(0, 3)),
        );
    }

    #[test]
    fn lex_keyword_entry() {
        assert_token(
            "entry",
            Token::new(TokenKind::Keyword(Keyword::Entry), Span::new(0, 5)),
        );
    }

    #[test]
    fn lex_identifier_entry() {
        assert_token(
            "Entry",
            Token::new(TokenKind::Identifier("Entry"), Span::new(0, 5)),
        );
    }

    #[test]
    fn lex_integer_single_digit() {
        assert_token("1", Token::new(TokenKind::Integer("1"), Span::new(0, 1)));
    }

    #[test]
    fn lex_integer_multiple_digits() {
        assert_token(
            "123",
            Token::new(TokenKind::Integer("123"), Span::new(0, 3)),
        );
    }

    #[test]
    fn lex_string_empty() {
        assert_token(
            r#""""#,
            Token::new(TokenKind::String(r#""""#), Span::new(0, 2)),
        );
    }

    #[test]
    fn lex_string_simple() {
        assert_token(
            r#""foo""#,
            Token::new(TokenKind::String(r#""foo""#), Span::new(0, 5)),
        );
    }

    #[test]
    fn lex_string_escaped_quote() {
        assert_token(
            r#""foo \"bar\"!""#,
            Token::new(TokenKind::String(r#""foo \"bar\"!""#), Span::new(0, 14)),
        );
    }

    #[test]
    fn lex_string_escaped_backslash() {
        assert_token(
            r#""foo\\bar""#,
            Token::new(TokenKind::String(r#""foo\\bar""#), Span::new(0, 10)),
        );
    }

    #[test]
    fn lex_colon() {
        assert_token(":", Token::new(TokenKind::Colon, Span::new(0, 1)));
    }

    #[test]
    fn lex_open_brace() {
        assert_token(
            "{",
            Token::new(TokenKind::OpenDelim(Delim::Brace), Span::new(0, 1)),
        );
    }

    #[test]
    fn lex_close_brace() {
        assert_token(
            "}",
            Token::new(TokenKind::CloseDelim(Delim::Brace), Span::new(0, 1)),
        );
    }

    #[test]
    fn lex_multiple_tokens_ignores_whitespace() {
        assert_tokens(
            r#"GET "example.com/"
GET "example.com/""#,
            &[
                Token::new(TokenKind::HttpMethod(HttpMethod::Get), Span::new(0, 3)),
                Token::new(TokenKind::String(r#""example.com/""#), Span::new(4, 18)),
                Token::new(TokenKind::HttpMethod(HttpMethod::Get), Span::new(19, 22)),
                Token::new(TokenKind::String(r#""example.com/""#), Span::new(23, 37)),
            ],
        );
    }

    fn assert_token(input: &str, expected: Token) {
        assert_tokens(input, &[expected]);
    }

    fn assert_tokens(input: &str, expected: &[Token]) {
        let actual = lex(input).expect("Input should not result in an error");
        assert_eq!(actual.len(), expected.len());
        assert_eq!(actual, expected);
    }
}
