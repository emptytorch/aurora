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
            let skipped_newline = self.skip_whitespace();
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
            return Ok(Some(Token {
                kind,
                span,
                skipped_newline,
            }));
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
            "null" => TokenKind::Keyword(Keyword::Null),
            "GET" => TokenKind::HttpMethod(HttpMethod::Get),
            "POST" => TokenKind::HttpMethod(HttpMethod::Post),
            "PUT" => TokenKind::HttpMethod(HttpMethod::Put),
            _ => TokenKind::Identifier(text),
        }
    }

    fn skip_whitespace(&mut self) -> bool {
        let mut skipped_newline = false;
        while let Some(ch) = self.first() {
            if !ch.is_whitespace() {
                break;
            }

            if ch == '\n' {
                skipped_newline = true;
            }

            self.bump();
        }

        skipped_newline
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.first() {
            if ch == '\n' {
                break;
            }
            self.bump();
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
            Token {
                kind: TokenKind::Identifier("foo"),
                span: Span::new(0, 3),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_identifier_with_underscore() {
        assert_token(
            "foo_bar",
            Token {
                kind: TokenKind::Identifier("foo_bar"),
                span: Span::new(0, 7),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_identifier_with_leading_underscore() {
        assert_token(
            "_foobar",
            Token {
                kind: TokenKind::Identifier("_foobar"),
                span: Span::new(0, 7),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_identifier_with_digits() {
        assert_token(
            "foo123",
            Token {
                kind: TokenKind::Identifier("foo123"),
                span: Span::new(0, 6),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_identifier_get() {
        assert_token(
            "get",
            Token {
                kind: TokenKind::Identifier("get"),
                span: Span::new(0, 3),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_identifier_post() {
        assert_token(
            "post",
            Token {
                kind: TokenKind::Identifier("post"),
                span: Span::new(0, 4),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_identifier_put() {
        assert_token(
            "put",
            Token {
                kind: TokenKind::Identifier("put"),
                span: Span::new(0, 3),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_http_method_get() {
        assert_token(
            "GET",
            Token {
                kind: TokenKind::HttpMethod(HttpMethod::Get),
                span: Span::new(0, 3),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_http_method_post() {
        assert_token(
            "POST",
            Token {
                kind: TokenKind::HttpMethod(HttpMethod::Post),
                span: Span::new(0, 4),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_http_method_put() {
        assert_token(
            "PUT",
            Token {
                kind: TokenKind::HttpMethod(HttpMethod::Put),
                span: Span::new(0, 3),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_keyword_entry() {
        assert_token(
            "entry",
            Token {
                kind: TokenKind::Keyword(Keyword::Entry),
                span: Span::new(0, 5),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_keyword_const() {
        assert_token(
            "const",
            Token {
                kind: TokenKind::Keyword(Keyword::Const),
                span: Span::new(0, 5),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_keyword_null() {
        assert_token(
            "null",
            Token {
                kind: TokenKind::Keyword(Keyword::Null),
                span: Span::new(0, 4),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_identifier_entry() {
        assert_token(
            "Entry",
            Token {
                kind: TokenKind::Identifier("Entry"),
                span: Span::new(0, 5),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_identifier_const() {
        assert_token(
            "Const",
            Token {
                kind: TokenKind::Identifier("Const"),
                span: Span::new(0, 5),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_identifier_null() {
        assert_token(
            "NULL",
            Token {
                kind: TokenKind::Identifier("NULL"),
                span: Span::new(0, 4),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_integer_single_digit() {
        assert_token(
            "1",
            Token {
                kind: TokenKind::Integer("1"),
                span: Span::new(0, 1),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_integer_multiple_digits() {
        assert_token(
            "123",
            Token {
                kind: TokenKind::Integer("123"),
                span: Span::new(0, 3),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_float_single_decimal() {
        assert_token(
            "0.0",
            Token {
                kind: TokenKind::Float("0.0"),
                span: Span::new(0, 3),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_float_multiple_decimals() {
        assert_token(
            "1.23",
            Token {
                kind: TokenKind::Float("1.23"),
                span: Span::new(0, 4),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_float() {
        assert_token(
            "123.456",
            Token {
                kind: TokenKind::Float("123.456"),
                span: Span::new(0, 7),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_string_empty() {
        assert_token(
            r#""""#,
            Token {
                kind: TokenKind::String(vec![]),
                span: Span::new(0, 2),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_string_simple() {
        assert_token(
            r#""foo""#,
            Token {
                kind: TokenKind::String(vec![TemplatePart::Literal("foo")]),
                span: Span::new(0, 5),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_string_escaped_quote() {
        assert_token(
            r#""foo \"bar\"!""#,
            Token {
                kind: TokenKind::String(vec![TemplatePart::Literal(r#"foo \"bar\"!"#)]),
                span: Span::new(0, 14),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_string_escaped_backslash() {
        assert_token(
            r#""foo\\bar""#,
            Token {
                kind: TokenKind::String(vec![TemplatePart::Literal(r#"foo\\bar"#)]),
                span: Span::new(0, 10),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_string_single_code_template_part() {
        assert_token(
            r#""{{foo}}""#,
            Token {
                kind: TokenKind::String(vec![TemplatePart::Code(vec![Token {
                    kind: TokenKind::Identifier("foo"),
                    span: Span::new(3, 6),
                    skipped_newline: false,
                }])]),
                span: Span::new(0, 9),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_string_single_code_template_part_leading_literal() {
        assert_token(
            r#""foo{{bar}}""#,
            Token {
                kind: TokenKind::String(vec![
                    TemplatePart::Literal("foo"),
                    TemplatePart::Code(vec![Token {
                        kind: TokenKind::Identifier("bar"),
                        span: Span::new(6, 9),
                        skipped_newline: false,
                    }]),
                ]),
                span: Span::new(0, 12),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_string_single_code_template_part_trailing_literal() {
        assert_token(
            r#""{{foo}}bar""#,
            Token {
                kind: TokenKind::String(vec![
                    TemplatePart::Code(vec![Token {
                        kind: TokenKind::Identifier("foo"),
                        span: Span::new(3, 6),
                        skipped_newline: false,
                    }]),
                    TemplatePart::Literal("bar"),
                ]),
                span: Span::new(0, 12),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_string_single_code_template_part_leading_and_trailing_literal() {
        assert_token(
            r#""foo{{bar}}baz""#,
            Token {
                kind: TokenKind::String(vec![
                    TemplatePart::Literal("foo"),
                    TemplatePart::Code(vec![Token {
                        kind: TokenKind::Identifier("bar"),
                        span: Span::new(6, 9),
                        skipped_newline: false,
                    }]),
                    TemplatePart::Literal("baz"),
                ]),
                span: Span::new(0, 15),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_string_multiple_code_template_part_leading_and_trailing_literal() {
        assert_token(
            r#""{{foo}}{{bar}}""#,
            Token {
                kind: TokenKind::String(vec![
                    TemplatePart::Code(vec![Token {
                        kind: TokenKind::Identifier("foo"),
                        span: Span::new(3, 6),
                        skipped_newline: false,
                    }]),
                    TemplatePart::Code(vec![Token {
                        kind: TokenKind::Identifier("bar"),
                        span: Span::new(10, 13),
                        skipped_newline: false,
                    }]),
                ]),
                span: Span::new(0, 16),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_colon() {
        assert_token(
            ":",
            Token {
                kind: TokenKind::Colon,
                span: Span::new(0, 1),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_comma() {
        assert_token(
            ",",
            Token {
                kind: TokenKind::Comma,
                span: Span::new(0, 1),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_eq() {
        assert_token(
            "=",
            Token {
                kind: TokenKind::Eq,
                span: Span::new(0, 1),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_open_brace() {
        assert_token(
            "{",
            Token {
                kind: TokenKind::Delim(Delim::OpenBrace),
                span: Span::new(0, 1),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_open_brack() {
        assert_token(
            "[",
            Token {
                kind: TokenKind::Delim(Delim::OpenBrack),
                span: Span::new(0, 1),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_close_brace() {
        assert_token(
            "}",
            Token {
                kind: TokenKind::Delim(Delim::CloseBrace),
                span: Span::new(0, 1),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_close_brack() {
        assert_token(
            "]",
            Token {
                kind: TokenKind::Delim(Delim::CloseBrack),
                span: Span::new(0, 1),
                skipped_newline: false,
            },
        );
    }

    #[test]
    fn lex_multiple_tokens_ignores_whitespace() {
        assert_tokens(
            r#"GET "example.com/"
GET "example.com/""#,
            &[
                Token {
                    kind: TokenKind::HttpMethod(HttpMethod::Get),
                    span: Span::new(0, 3),
                    skipped_newline: false,
                },
                Token {
                    kind: TokenKind::String(vec![TemplatePart::Literal("example.com/")]),
                    span: Span::new(4, 18),
                    skipped_newline: false,
                },
                Token {
                    kind: TokenKind::HttpMethod(HttpMethod::Get),
                    span: Span::new(19, 22),
                    skipped_newline: true,
                },
                Token {
                    kind: TokenKind::String(vec![TemplatePart::Literal("example.com/")]),
                    span: Span::new(23, 37),
                    skipped_newline: false,
                },
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
                Token {
                    kind: TokenKind::HttpMethod(HttpMethod::Get),
                    span: Span::new(21, 24),
                    skipped_newline: true,
                },
                Token {
                    kind: TokenKind::String(vec![TemplatePart::Literal("example.com/")]),
                    span: Span::new(25, 39),
                    skipped_newline: false,
                },
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
                Token {
                    kind: TokenKind::HttpMethod(HttpMethod::Get),
                    span: Span::new(26, 29),
                    skipped_newline: true,
                },
                Token {
                    kind: TokenKind::String(vec![TemplatePart::Literal("example.com/")]),
                    span: Span::new(30, 44),
                    skipped_newline: false,
                },
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
