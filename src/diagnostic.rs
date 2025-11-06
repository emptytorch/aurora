use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
}

#[derive(Debug)]
pub struct Label {
    pub message: String,
    pub span: Span,
    pub level: Level,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,
    pub level: Level,
    pub labels: Vec<Label>,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>, span: Span, level: Level) -> Self {
        Self {
            message: message.into(),
            span,
            level,
            labels: vec![],
        }
    }

    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self::new(message, span, Level::Error)
    }

    pub fn label(mut self, message: impl Into<String>, span: Span, level: Level) -> Self {
        let label = Label {
            message: message.into(),
            span,
            level,
        };
        self.labels.push(label);
        self
    }
}
