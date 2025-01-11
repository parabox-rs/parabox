use crate::source::Source;
use ecow::EcoString;
use std::fmt::{Debug, Display};
use std::ops::Range;
use std::rc::Rc;

/// A span of text in a source.
///
/// Contains the source and the span range.
#[derive(Clone)]
pub struct Span {
    source: Rc<dyn Source>,
    range: Range<usize>,
}

impl Span {
    /// Creates a new span from a source and a range.
    pub fn new(source: Rc<dyn Source>, range: Range<usize>) -> Self {
        Self { source, range }
    }

    /// Returns the source of the span.
    pub fn source(&self) -> Rc<dyn Source> {
        Rc::clone(&self.source)
    }

    /// Returns the range of the span.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    /// Returns the name of the source.
    pub fn name(&self) -> EcoString {
        self.source.name()
    }

    /// Returns the text of the span.
    pub fn text(&self) -> &str {
        &self.source.text()[self.range.clone()]
    }

    /// Returns the line and column of the span.
    pub fn locate(&self) -> (usize, usize) {
        self.source.locate(self.range.start)
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (line, column) = self.locate();
        write!(
            f,
            "{}:{}:{}: {}",
            self.name(),
            line + 1,
            column + 1,
            self.text()
        )
    }
}

/// An error that occurs during parsing.
///
/// Contains the error message and the span where the error occurred.
pub struct ParseError {
    span: Span,
    message: EcoString,
}

impl ParseError {
    pub(crate) fn new(span: Span, message: EcoString) -> Self {
        Self { span, message }
    }

    /// Returns the span where the error occurred.
    pub fn span(&self) -> Span {
        self.span.clone()
    }

    /// Returns the error message.
    pub fn message(&self) -> EcoString {
        self.message.clone()
    }
}

/// The number of lines to display before and after the error.
const DISPLAY_CONTEXT_LINES: usize = 3;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source = self.span.source();
        let range = self.span.range();

        let (line, column) = source.locate(range.start);

        let context_start = line.saturating_sub(DISPLAY_CONTEXT_LINES);
        let context_end = (line + DISPLAY_CONTEXT_LINES + 1).min(source.line_len());
        let context_range = context_start..context_end;

        let code_indent = context_range.end.to_string().len();

        write!(
            f,
            "error: {}\n  --> {}:{}:{}\n",
            self.message,
            source.name(),
            line + 1,
            column + 1,
        )?;

        macro_rules! write_line {
            ($before:expr, $after:expr) => {
                write!(f, "{:width$} │ {}\n", $before, $after, width = code_indent)?;
            };

            ($after:expr) => {
                write_line!("", $after);
            };

            () => {
                write_line!("", "");
            };
        }

        for context_line in context_range.clone() {
            if context_line == line {
                write_line!();
            }

            write_line!(context_line + 1, source.line(context_line).unwrap());

            if context_line == line {
                write_line!(format!(
                    "{}{}",
                    " ".repeat(column),
                    "^".repeat(range.end - range.start)
                ));
            }
        }

        Ok(())
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

/// The result of parsing, just an alias to [Result] with error type
/// [ParseError].
pub type ParseResult<T> = Result<T, ParseError>;
