use crate::kind::SyntaxKind;
use ecow::EcoString;
use std::ops::Range;
use unscanny::Scanner;

pub struct Lexer<'s> {
    s: Scanner<'s>,
    start: usize,
}

impl<'s> Lexer<'s> {
    pub fn new(text: &'s str) -> Self {
        Self {
            s: Scanner::new(text),
            start: 0,
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn cursor(&self) -> usize {
        self.s.cursor()
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.start..self.s.cursor()
    }

    pub fn text(&self) -> &'s str {
        self.s.from(self.start)
    }
}

impl Lexer<'_> {
    pub fn next(&mut self) -> LexResult<SyntaxKind> {
        self.start = self.s.cursor();

        match self.s.eat() {
            Some(',') => Ok(SyntaxKind::Comma),
            Some('(') => Ok(SyntaxKind::LeftParen),
            Some(')') => Ok(SyntaxKind::RightParen),
            Some('/') if self.s.eat_if('/') => {
                self.s.eat_while(|_: char| true);
                Ok(SyntaxKind::Comment)
            }
            Some(c) if c.is_whitespace() => {
                self.s.eat_whitespace();
                Ok(SyntaxKind::Empty)
            }
            None => Ok(SyntaxKind::Eol),

            Some(c) if c.is_digit(10) => {
                self.s.eat_while(|c: char| c.is_digit(10));
                Ok(SyntaxKind::Integer)
            }

            Some('#') => {
                self.s.eat_while(is_ident_char);
                Ok(SyntaxKind::Ident)
            }

            Some(c) if c.is_alphabetic() => {
                self.s.eat_while(char::is_alphabetic);
                keyword(self.text()).ok_or_else(|| self.unexpected_keyword())
            }

            _ => Err(self.unexpected_char()),
        }
    }
}

impl Lexer<'_> {
    pub fn error(&self, message: String) -> LexError {
        LexError::new(self.range(), message.into())
    }

    pub fn unexpected_char(&self) -> LexError {
        self.error(format!("unexpected character `{}`", self.text()))
    }

    pub fn unexpected_keyword(&self) -> LexError {
        self.error(format!("unexpected keyword `{}`", self.text()))
    }
}

#[inline]
pub fn is_newline_char(character: char) -> bool {
    matches!(
        character,
        // Line Feed, Vertical Tab, Form Feed, Carriage Return.
        '\n' | '\x0B' | '\x0C' | '\r' |
        // Next Line, Line Separator, Paragraph Separator.
        '\u{0085}' | '\u{2028}' | '\u{2029}'
    )
}

#[inline]
pub fn is_ident_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn keyword(text: &str) -> Option<SyntaxKind> {
    match text.to_lowercase().as_str() {
        "define" => Some(SyntaxKind::Define),
        "place" => Some(SyntaxKind::Place),
        "push" => Some(SyntaxKind::Push),
        "expect" => Some(SyntaxKind::Expect),
        "wall" => Some(SyntaxKind::Wall),
        "box" => Some(SyntaxKind::Box),
        "alias" => Some(SyntaxKind::Alias),
        "infinity" => Some(SyntaxKind::Infinity),
        "epsilon" => Some(SyntaxKind::Epsilon),
        "void" => Some(SyntaxKind::Void),
        "size" => Some(SyntaxKind::Size),
        "ref" => Some(SyntaxKind::Ref),
        "solid" => Some(SyntaxKind::Solid),
        "at" => Some(SyntaxKind::At),
        "in" => Some(SyntaxKind::In),
        "orphan" => Some(SyntaxKind::Orphan),
        "north" => Some(SyntaxKind::North),
        "south" => Some(SyntaxKind::South),
        "east" => Some(SyntaxKind::East),
        "west" => Some(SyntaxKind::West),
        "moved" => Some(SyntaxKind::Moved),
        "static" => Some(SyntaxKind::Static),
        _ => None,
    }
}

pub struct LexError {
    range: Range<usize>,
    message: EcoString,
}

impl LexError {
    pub fn new(range: Range<usize>, message: EcoString) -> Self {
        Self { range, message }
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn message(&self) -> EcoString {
        self.message.clone()
    }
}

pub type LexResult<T> = Result<T, LexError>;
