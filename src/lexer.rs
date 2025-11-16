use crate::{
    diagnostic::{Diagnostic, Level},
    span::Span,
    token::{Delim, HttpMethod, Keyword, TemplatePart, Token, TokenKind},
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
        loop {
            self.skip_whitespace();
            if Some('#') == self.first() {
                self.skip_comment();
                continue;
            }

            let Some(first) = self.first() else {
                return Ok(None);
            };

            let start = self.pos;
            self.bump();

            let kind = match first {
                ':' => TokenKind::Colon,
                ',' => TokenKind::Comma,
                '=' => TokenKind::Eq,
                '{' => TokenKind::Delim(Delim::OpenBrace),
                '[' => TokenKind::Delim(Delim::OpenBrack),
                '}' => TokenKind::Delim(Delim::CloseBrace),
                ']' => TokenKind::Delim(Delim::CloseBrack),
                '"' => self.string(start)?,
                _ if first.is_ascii_digit() => self.number(start),
                _ if first.is_alphabetic() || first == '_' => self.identifier(start),
                _ => {
                    return Err(Diagnostic::error(
                        "Unrecognized character",
                        Span::new(start, start),
                    )
                    .primary_label("I don't know what to do with this character", Level::Error));
                }
            };

            let span = Span::new(start, self.pos);
            return Ok(Some(Token::new(kind, span)));
        }
    }

    fn string(&mut self, start: usize) -> Result<TokenKind<'input>, Diagnostic> {
        let mut parts = vec![];
        let mut chunk_start = self.pos;

        while let Some(ch) = self.next() {
            match ch {
                '"' => {
                    if chunk_start < self.pos - 1 {
                        parts.push(TemplatePart::Literal(
                            &self.input[chunk_start..self.pos - 1],
                        ));
                    }
                    return Ok(TokenKind::String(parts));
                }

                '{' if self.first() == Some('{') => {
                    if chunk_start < self.pos - 1 {
                        parts.push(TemplatePart::Literal(
                            &self.input[chunk_start..self.pos - 1],
                        ));
                    }

                    self.bump();
                    let mut tokens = vec![];

                    loop {
                        match self.first() {
                            None | Some('"') => {
                                return Err(Diagnostic::error(
                                    "Unterminated template",
                                    Span::new(self.pos, self.pos),
                                )
                                .primary_label("I was expecting `}}` here", Level::Error));
                            }
                            Some('}') if self.second() == Some('}') => {
                                self.bump();
                                self.bump();
                                break;
                            }
                            _ => {
                                let Some(token) = self.next_token()? else {
                                    return Err(Diagnostic::error(
                                        "Unterminated template",
                                        Span::new(self.pos, self.pos),
                                    )
                                    .primary_label("I was expecting `}}` here", Level::Error));
                                };
                                tokens.push(token);
                            }
                        }
                    }

                    parts.push(TemplatePart::Code(tokens));
                    chunk_start = self.pos;
                }
                '\\' => {
                    self.bump();
                }
                _ => {}
            }
        }

        Err(
            Diagnostic::error("Unterminated string literal", Span::new(start, self.pos))
                .primary_label(
                    "I never found the closing quote for this string",
                    Level::Error,
                ),
        )
    }

    fn number(&mut self, start: usize) -> TokenKind<'input> {
        fn eat_digits<'input>(l: &mut Lexer<'input>) {
            while let Some(ch) = l.first() {
                if !ch.is_ascii_digit() {
                    break;
                }
                l.bump();
            }
        }

        eat_digits(self);

        let is_float = if matches!(self.first(), Some('.')) {
            self.bump();
            eat_digits(self);
            true
        } else {
            false
        };

        let text = &self.input[start..self.pos];
        if is_float {
            TokenKind::Float(text)
        } else {
            TokenKind::Integer(text)
        }
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
            "const" => TokenKind::Keyword(Keyword::Const),
            "GET" => TokenKind::HttpMethod(HttpMethod::Get),
            "POST" => TokenKind::HttpMethod(HttpMethod::Post),
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

    fn skip_comment(&mut self) {
        while let Some(ch) = self.next() {
            if ch == '\n' {
                break;
            }
        }
    }

    fn first(&mut self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn second(&mut self) -> Option<char> {
        self.input[self.pos..].chars().nth(1)
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
    fn lex_identifier_post() {
        assert_token(
            "post",
            Token::new(TokenKind::Identifier("post"), Span::new(0, 4)),
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
    fn lex_http_method_post() {
        assert_token(
            "POST",
            Token::new(TokenKind::HttpMethod(HttpMethod::Post), Span::new(0, 4)),
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
    fn lex_keyword_const() {
        assert_token(
            "const",
            Token::new(TokenKind::Keyword(Keyword::Const), Span::new(0, 5)),
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
    fn lex_identifier_const() {
        assert_token(
            "Const",
            Token::new(TokenKind::Identifier("Const"), Span::new(0, 5)),
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
    fn lex_float_single_decimal() {
        assert_token("0.0", Token::new(TokenKind::Float("0.0"), Span::new(0, 3)));
    }

    #[test]
    fn lex_float_multiple_decimals() {
        assert_token(
            "1.23",
            Token::new(TokenKind::Float("1.23"), Span::new(0, 4)),
        );
    }

    #[test]
    fn lex_float() {
        assert_token(
            "123.456",
            Token::new(TokenKind::Float("123.456"), Span::new(0, 7)),
        );
    }

    #[test]
    fn lex_string_empty() {
        assert_token(
            r#""""#,
            Token::new(TokenKind::String(vec![]), Span::new(0, 2)),
        );
    }

    #[test]
    fn lex_string_simple() {
        assert_token(
            r#""foo""#,
            Token::new(
                TokenKind::String(vec![TemplatePart::Literal("foo")]),
                Span::new(0, 5),
            ),
        );
    }

    #[test]
    fn lex_string_escaped_quote() {
        assert_token(
            r#""foo \"bar\"!""#,
            Token::new(
                TokenKind::String(vec![TemplatePart::Literal(r#"foo \"bar\"!"#)]),
                Span::new(0, 14),
            ),
        );
    }

    #[test]
    fn lex_string_escaped_backslash() {
        assert_token(
            r#""foo\\bar""#,
            Token::new(
                TokenKind::String(vec![TemplatePart::Literal(r#"foo\\bar"#)]),
                Span::new(0, 10),
            ),
        );
    }

    #[test]
    fn lex_string_single_code_template_part() {
        assert_token(
            r#""{{foo}}""#,
            Token::new(
                TokenKind::String(vec![TemplatePart::Code(vec![Token::new(
                    TokenKind::Identifier("foo"),
                    Span::new(3, 6),
                )])]),
                Span::new(0, 9),
            ),
        );
    }

    #[test]
    fn lex_string_single_code_template_part_leading_literal() {
        assert_token(
            r#""foo{{bar}}""#,
            Token::new(
                TokenKind::String(vec![
                    TemplatePart::Literal("foo"),
                    TemplatePart::Code(vec![Token::new(
                        TokenKind::Identifier("bar"),
                        Span::new(6, 9),
                    )]),
                ]),
                Span::new(0, 12),
            ),
        );
    }

    #[test]
    fn lex_string_single_code_template_part_trailing_literal() {
        assert_token(
            r#""{{foo}}bar""#,
            Token::new(
                TokenKind::String(vec![
                    TemplatePart::Code(vec![Token::new(
                        TokenKind::Identifier("foo"),
                        Span::new(3, 6),
                    )]),
                    TemplatePart::Literal("bar"),
                ]),
                Span::new(0, 12),
            ),
        );
    }

    #[test]
    fn lex_string_single_code_template_part_leading_and_trailing_literal() {
        assert_token(
            r#""foo{{bar}}baz""#,
            Token::new(
                TokenKind::String(vec![
                    TemplatePart::Literal("foo"),
                    TemplatePart::Code(vec![Token::new(
                        TokenKind::Identifier("bar"),
                        Span::new(6, 9),
                    )]),
                    TemplatePart::Literal("baz"),
                ]),
                Span::new(0, 15),
            ),
        );
    }

    #[test]
    fn lex_string_multiple_code_template_part_leading_and_trailing_literal() {
        assert_token(
            r#""{{foo}}{{bar}}""#,
            Token::new(
                TokenKind::String(vec![
                    TemplatePart::Code(vec![Token::new(
                        TokenKind::Identifier("foo"),
                        Span::new(3, 6),
                    )]),
                    TemplatePart::Code(vec![Token::new(
                        TokenKind::Identifier("bar"),
                        Span::new(10, 13),
                    )]),
                ]),
                Span::new(0, 16),
            ),
        );
    }

    #[test]
    fn lex_colon() {
        assert_token(":", Token::new(TokenKind::Colon, Span::new(0, 1)));
    }

    #[test]
    fn lex_comma() {
        assert_token(",", Token::new(TokenKind::Comma, Span::new(0, 1)));
    }

    #[test]
    fn lex_eq() {
        assert_token("=", Token::new(TokenKind::Eq, Span::new(0, 1)));
    }

    #[test]
    fn lex_open_brace() {
        assert_token(
            "{",
            Token::new(TokenKind::Delim(Delim::OpenBrace), Span::new(0, 1)),
        );
    }

    #[test]
    fn lex_open_brack() {
        assert_token(
            "[",
            Token::new(TokenKind::Delim(Delim::OpenBrack), Span::new(0, 1)),
        );
    }

    #[test]
    fn lex_close_brace() {
        assert_token(
            "}",
            Token::new(TokenKind::Delim(Delim::CloseBrace), Span::new(0, 1)),
        );
    }

    #[test]
    fn lex_close_brack() {
        assert_token(
            "]",
            Token::new(TokenKind::Delim(Delim::CloseBrack), Span::new(0, 1)),
        );
    }

    #[test]
    fn lex_multiple_tokens_ignores_whitespace() {
        assert_tokens(
            r#"GET "example.com/"
GET "example.com/""#,
            &[
                Token::new(TokenKind::HttpMethod(HttpMethod::Get), Span::new(0, 3)),
                Token::new(
                    TokenKind::String(vec![TemplatePart::Literal("example.com/")]),
                    Span::new(4, 18),
                ),
                Token::new(TokenKind::HttpMethod(HttpMethod::Get), Span::new(19, 22)),
                Token::new(
                    TokenKind::String(vec![TemplatePart::Literal("example.com/")]),
                    Span::new(23, 37),
                ),
            ],
        );
    }

    #[test]
    fn lex_skips_full_line_comment() {
        assert_tokens(
            r#"
# This is a comment
GET "example.com/""#,
            &[
                Token::new(TokenKind::HttpMethod(HttpMethod::Get), Span::new(21, 24)),
                Token::new(
                    TokenKind::String(vec![TemplatePart::Literal("example.com/")]),
                    Span::new(25, 39),
                ),
            ],
        );
    }

    #[test]
    fn lex_skips_multiple_comments_and_whitespace() {
        assert_tokens(
            r#"
# comment 1
# comment 2

GET "example.com/"
# comment 3
"#,
            &[
                Token::new(TokenKind::HttpMethod(HttpMethod::Get), Span::new(26, 29)),
                Token::new(
                    TokenKind::String(vec![TemplatePart::Literal("example.com/")]),
                    Span::new(30, 44),
                ),
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
