use std::{fmt, path::Path};

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

    pub fn primary_label(self, message: impl Into<String>, level: Level) -> Self {
        let span = self.span;
        self.label(message, span, level)
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

#[derive(Debug, Clone, Copy)]
pub enum RenderStyle {
    Styled,
    Plain,
}

pub fn dump<W: fmt::Write>(
    input: &str,
    path: &Path,
    diagnostic: &Diagnostic,
    style: RenderStyle,
    w: &mut W,
) -> fmt::Result {
    let mut annotations: Vec<annotate_snippets::Annotation> = vec![];
    let mut primary_found = false;
    for label in &diagnostic.labels {
        let annotation_kind = if !primary_found && label.span == diagnostic.span {
            primary_found = true;
            annotate_snippets::AnnotationKind::Primary
        } else {
            annotate_snippets::AnnotationKind::Context
        };

        annotations.push(
            annotation_kind
                .span(label.span.start..label.span.end)
                .label(&label.message),
        );
    }

    if !primary_found {
        annotations.insert(
            0,
            annotate_snippets::AnnotationKind::Primary
                .span(diagnostic.span.start..diagnostic.span.end)
                .label("here"),
        );
    }

    let report = &[annotate_snippets::Level::ERROR
        .primary_title(&diagnostic.message)
        .element(
            annotate_snippets::Snippet::source(input)
                .line_start(1)
                .path(path.to_string_lossy())
                .annotations(annotations),
        )];

    let renderer = match style {
        RenderStyle::Styled => annotate_snippets::Renderer::styled()
            .decor_style(annotate_snippets::renderer::DecorStyle::Unicode),
        RenderStyle::Plain => annotate_snippets::Renderer::plain(),
    };

    write!(w, "{}", renderer.render(report))
}
