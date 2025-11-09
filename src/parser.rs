use crate::{
    ast::{
        Entry, EntryItem, EntryItemKind, Expr, ExprKind, HttpMethod, Item, ItemKind, Name, Request,
    },
    diagnostic::{Diagnostic, Level},
    lexer,
    span::Span,
    token::{self, Delim, Keyword, Token, TokenKind},
};

pub fn parse<'input>(input: &'input str) -> Result<Vec<Item<'input>>, Diagnostic> {
    let tokens = lexer::lex(input)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

struct Parser<'input> {
    tokens: Vec<Token<'input>>,
    pos: usize,
}

impl<'input> Parser<'input> {
    fn new(tokens: Vec<Token<'input>>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse(&mut self) -> Result<Vec<Item<'input>>, Diagnostic> {
        let mut items = vec![];
        while self.peek().is_some() {
            let item = self.parse_item()?;
            items.push(item);
        }
        Ok(items)
    }

    fn parse_item(&mut self) -> Result<Item<'input>, Diagnostic> {
        if let Some(span) = self.eat_keyword(Keyword::Entry) {
            return self.parse_entry(span);
        }

        Err(Diagnostic::error("Expected item", self.peek_span()).label(
            "I was expecting an item here",
            self.peek_span(),
            Level::Error,
        ))
    }

    fn parse_entry(&mut self, entry_span: Span) -> Result<Item<'input>, Diagnostic> {
        let name = self.parse_name().ok_or(
            Diagnostic::error("Expected identifier", self.peek_span()).label(
                "I was expecting a name here",
                self.peek_span(),
                Level::Error,
            ),
        )?;

        _ = self.expect_delim(Delim::OpenBrace)?;
        let mut entry_items = vec![];
        while let Some(item) = self.opt_parse_entry_item()? {
            entry_items.push(item);
        }
        let close_span = self.expect_delim(Delim::CloseBrace)?;
        let span = entry_span.to(close_span);
        Ok(Item {
            kind: ItemKind::Entry(Entry {
                name,
                body: entry_items,
            }),
            span,
        })
    }

    fn opt_parse_entry_item(&mut self) -> Result<Option<EntryItem<'input>>, Diagnostic> {
        match self.peek() {
            Some(&Token {
                kind: TokenKind::HttpMethod(token::HttpMethod::Get),
                span: method_span,
            }) => {
                self.bump();
                let url = self.parse_expr()?;
                let url_span = url.span;
                Ok(Some(EntryItem {
                    kind: EntryItemKind::Request(Request {
                        method: HttpMethod::Get,
                        url,
                    }),
                    span: method_span.to(url_span),
                }))
            }
            _ => Ok(None),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr<'input>, Diagnostic> {
        match self.opt_parse_expr()? {
            Some(expr) => Ok(expr),
            None => Err(
                Diagnostic::error("Expected expression", self.peek_span()).label(
                    "I was expecting an expression here",
                    self.peek_span(),
                    Level::Error,
                ),
            ),
        }
    }

    fn opt_parse_expr(&mut self) -> Result<Option<Expr<'input>>, Diagnostic> {
        match self.peek() {
            Some(&Token {
                kind: TokenKind::String(s),
                span,
            }) => {
                self.bump();
                Ok(Some(Expr {
                    kind: ExprKind::StringLiteral(s),
                    span,
                }))
            }
            _ => Ok(None),
        }
    }

    fn parse_name(&mut self) -> Option<Name<'input>> {
        if let Some(&Token {
            kind: TokenKind::Identifier(text),
            span,
        }) = self.peek()
        {
            self.bump();
            Some(Name { text, span })
        } else {
            None
        }
    }

    fn eat_keyword(&mut self, kw: Keyword) -> Option<Span> {
        if let Some(&Token {
            kind: TokenKind::Keyword(kw2),
            span,
        }) = self.peek()
            && kw == kw2
        {
            self.bump();
            Some(span)
        } else {
            None
        }
    }

    fn expect_delim(&mut self, delim: Delim) -> Result<Span, Diagnostic> {
        if let Some(&Token {
            kind: TokenKind::Delim(delim2),
            span,
        }) = self.peek()
            && delim == delim2
        {
            self.bump();
            Ok(span)
        } else {
            let label_message = if delim.is_open() {
                format!("I was expecting an opening delimiter `{delim}` here")
            } else {
                format!("I was expecting a closing delimiter `{delim}` here")
            };

            Err(
                Diagnostic::error("Expected delimiter", self.peek_span()).label(
                    label_message,
                    self.peek_span(),
                    Level::Error,
                ),
            )
        }
    }

    fn peek(&self) -> Option<&Token<'input>> {
        self.tokens.get(self.pos)
    }

    fn peek_span(&self) -> Span {
        if let Some(token) = self.peek() {
            token.span
        } else if let Some(last) = self.tokens.last() {
            last.span
        } else {
            Span::new(0, 0)
        }
    }

    fn bump(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }
}
