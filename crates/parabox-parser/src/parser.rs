use crate::command::{Assertion, Command, MetaProtoType, Operation};
use crate::error::{ParseError, ParseResult, Span};
use crate::kind::SyntaxKind;
use crate::lexer::{LexError, LexResult, Lexer};
use crate::source::Source;
use ecow::EcoString;
use parabox::{Direction, Size};
use std::rc::Rc;

/// Parses a source into a list of commands.
pub fn parse(source: Rc<dyn Source>) -> ParseResult<Vec<SpannedCommand>> {
    let mut spanned_commands = vec![];

    for line in 0..source.line_len() {
        let text = source.line(line).unwrap();
        let range = source.line_range(line).unwrap();
        let commands = match parse_line(text) {
            Ok(commands) => commands,
            Err(e) => {
                let error_range = e.range();
                let range = (range.start + error_range.start)..(range.start + error_range.end);
                return Err(ParseError::new(Span::new(source, range), e.message()));
            }
        };

        for command in commands {
            spanned_commands.push(SpannedCommand::new(
                command,
                Span::new(source.clone(), range.clone()),
            ));
        }
    }

    Ok(spanned_commands)
}

fn parse_line(text: &str) -> LexResult<Vec<Command>> {
    let mut parser = Parser::new(text);
    let mut commands = vec![];

    while !parser.peek()?.is_eof() {
        match parser.next()? {
            SyntaxKind::Define => {
                commands.append(&mut define(&mut parser)?);
            }
            SyntaxKind::Place => {
                commands.push(place(&mut parser)?);
            }
            SyntaxKind::Push => {
                commands.push(push(&mut parser)?);
            }
            SyntaxKind::Expect => {
                commands.push(expect(&mut parser)?);
            }
            _ => {
                return Err(parser.expected("statement keyword"));
            }
        };
    }

    Ok(commands)
}

fn define(parser: &mut Parser) -> LexResult<Vec<Command>> {
    let proto = parser.expect_proto()?;
    let block = parser.expect_ident()?;

    let mut reference = None;
    let mut size = None;
    let mut solid = false;

    while !parser.peek()?.is_eof() {
        match parser.next()? {
            SyntaxKind::Size => {
                if !proto.proto_needs_size() {
                    return Err(parser.unexpected("`size` keyword"));
                }

                if solid {
                    return Err(parser.conflict("`size` keyword", "`solid` keyword"));
                }

                if size.is_some() {
                    return Err(parser.multiple("`size` keywords"));
                }

                size = Some(parser.expect_size()?);
            }
            SyntaxKind::Ref => {
                if !proto.proto_needs_reference() {
                    return Err(parser.unexpected("`ref` keyword"));
                }

                if reference.is_some() {
                    return Err(parser.multiple("`ref` keywords"));
                }

                reference = Some(parser.expect_ident()?);
            }
            SyntaxKind::Solid => {
                if !matches!(proto, SyntaxKind::Box) {
                    return Err(parser.unexpected("`solid` keyword"));
                }

                if size.is_some() {
                    return Err(parser.conflict("`solid` keyword", "`size` keyword"));
                }

                if solid {
                    return Err(parser.multiple("`solid` keywords"));
                }

                solid = true;
            }
            _ => {
                return Err(parser.expected("`size`, `ref` or `solid`"));
            }
        }
    }

    let mut size = size.unwrap_or_default();

    if solid {
        size = (1, 1);
    }

    let proto = match proto {
        SyntaxKind::Wall => MetaProtoType::Wall,
        SyntaxKind::Box => MetaProtoType::Box { size },
        SyntaxKind::Alias => MetaProtoType::Alias {
            reference: reference.ok_or_else(|| parser.missing("`ref` keyword"))?,
        },
        SyntaxKind::Infinity => MetaProtoType::Infinity {
            reference: reference.ok_or_else(|| parser.missing("`ref` keyword"))?,
        },
        SyntaxKind::Epsilon => MetaProtoType::Epsilon {
            reference: reference.ok_or_else(|| parser.missing("`ref` keyword"))?,
            size,
        },
        SyntaxKind::Void => MetaProtoType::Void { size },
        _ => unreachable!(),
    };

    let statement = Command::define(block.clone(), proto);

    let result = if solid {
        let interior: EcoString = format!("{}::interior", block).into();
        let interior_proto = MetaProtoType::Wall;

        vec![
            statement,
            Command::define(interior.clone(), interior_proto),
            Command::place(interior.clone(), Some(block), (0, 0)),
        ]
    } else {
        vec![statement]
    };

    Ok(result)
}

fn place(parser: &mut Parser) -> LexResult<Command> {
    let block = parser.expect_ident()?;

    let mut container = None;
    let mut pos = None;
    let mut orphan = false;

    while !parser.peek()?.is_eof() {
        match parser.next()? {
            SyntaxKind::At => {
                if orphan {
                    return Err(parser.conflict("`at` keyword", "`orphan` keyword"));
                }
                if pos.is_none() {
                    pos = Some(parser.expect_size()?);
                } else {
                    return Err(parser.multiple("`at` keywords"));
                }
            }
            SyntaxKind::In => {
                if orphan {
                    return Err(parser.conflict("`in` keyword", "`orphan` keyword"));
                }
                if container.is_none() {
                    container = Some(parser.expect_ident()?);
                } else {
                    return Err(parser.multiple("`in` keywords"));
                }
            }
            SyntaxKind::Orphan => {
                if container.is_some() {
                    return Err(parser.conflict("`orphan` keyword", "`in` keyword"));
                }
                if pos.is_some() {
                    return Err(parser.conflict("`orphan` keyword", "`at` keyword"));
                }
                if !orphan {
                    orphan = true;
                } else {
                    return Err(parser.multiple("`orphan` keywords"));
                }
            }
            _ => {
                return Err(parser.expected("`size` or `ref`"));
            }
        }
    }

    let (container, pos) = if orphan {
        (None, (0, 0))
    } else {
        let container = container.ok_or_else(|| parser.missing("`in` keyword"))?;
        let pos = pos.ok_or_else(|| parser.missing("`at` keyword"))?;
        (Some(container), pos)
    };

    Ok(Command::place(block, container, pos))
}

fn push(parser: &mut Parser) -> LexResult<Command> {
    let block = parser.expect_ident()?;
    let direction = parser.expect_direction()?;
    let assertion = match parser.peek()? {
        SyntaxKind::Moved => {
            parser.next()?;
            Assertion::Moved
        }
        SyntaxKind::Static => {
            parser.next()?;
            Assertion::Static
        }
        _ => Assertion::None,
    };

    Ok(Command::push(block, direction, assertion))
}

fn expect(parser: &mut Parser) -> LexResult<Command> {
    let statement = place(parser)?;
    let position = match statement.operation {
        Operation::Place(pos) => pos,
        _ => unreachable!(),
    };
    Ok(Command::expect(
        statement.block,
        position.container,
        position.pos,
    ))
}

struct Parser<'s> {
    lexer: Lexer<'s>,
    peeked: Option<SyntaxKind>,
}

impl<'s> Parser<'s> {
    pub fn new(text: &'s str) -> Self {
        Self {
            lexer: Lexer::new(text),
            peeked: None,
        }
    }

    pub fn next(&mut self) -> LexResult<SyntaxKind> {
        if let Some(kind) = self.peeked.take() {
            return Ok(kind);
        }

        loop {
            let next = self.lexer.next()?;
            if next.is_skipped() {
                continue;
            }
            return Ok(next);
        }
    }

    pub fn peek(&mut self) -> LexResult<SyntaxKind> {
        let kind = self.next()?;
        self.peeked = Some(kind);
        Ok(kind)
    }

    pub fn text(&self) -> &'s str {
        self.lexer.text()
    }

    pub fn cursor(&self) -> usize {
        self.lexer.cursor()
    }
}

impl Parser<'_> {
    pub fn expect(&mut self, kind: SyntaxKind) -> LexResult<()> {
        let next = self.next()?;
        if next == kind {
            Ok(())
        } else {
            Err(self.expected(kind))
        }
    }

    pub fn expect_integer(&mut self) -> LexResult<usize> {
        self.expect(SyntaxKind::Integer)?;
        Ok(self.text().parse().unwrap())
    }

    pub fn expect_size(&mut self) -> LexResult<Size> {
        self.expect(SyntaxKind::LeftParen)?;
        let x = self.expect_integer()?;
        self.expect(SyntaxKind::Comma)?;
        let y = self.expect_integer()?;
        self.expect(SyntaxKind::RightParen)?;
        Ok((x, y))
    }

    pub fn expect_ident(&mut self) -> LexResult<EcoString> {
        self.expect(SyntaxKind::Ident)?;
        Ok(self.text()[1..].into())
    }

    pub fn expect_proto(&mut self) -> LexResult<SyntaxKind> {
        let next = self.next()?;
        if next.is_proto() {
            Ok(next)
        } else {
            Err(self.expected("proto type keyword"))
        }
    }

    pub fn expect_direction(&mut self) -> LexResult<Direction> {
        let next = self.next()?;
        match next {
            SyntaxKind::North => Ok(Direction::North),
            SyntaxKind::South => Ok(Direction::South),
            SyntaxKind::East => Ok(Direction::East),
            SyntaxKind::West => Ok(Direction::West),
            _ => Err(self.expected("direction")),
        }
    }
}

impl Parser<'_> {
    pub fn error(&self, message: String) -> LexError {
        self.lexer.error(message)
    }

    pub fn expected(&self, expect: impl ToString) -> LexError {
        self.error(format!("expected {}", expect.to_string()))
    }

    pub fn multiple(&self, multiple: &str) -> LexError {
        self.error(format!("multiple {}", multiple))
    }

    pub fn conflict(&self, source: &str, conflict: &str) -> LexError {
        self.error(format!("conflicting {} and {}", source, conflict))
    }

    pub fn missing(&self, missing: &str) -> LexError {
        self.error(format!("missing {}", missing))
    }

    pub fn unexpected(&self, unexpected: &str) -> LexError {
        self.error(format!("unexpected {}", unexpected))
    }
}

/// A command with a span.
pub struct SpannedCommand {
    command: Command,
    span: Span,
}

impl SpannedCommand {
    /// Creates a new spanned command.
    pub fn new(command: Command, span: Span) -> Self {
        Self { command, span }
    }

    /// Returns the command.
    pub fn command(&self) -> &Command {
        &self.command
    }

    /// Returns the span.
    pub fn span(&self) -> &Span {
        &self.span
    }
}
