use std::fmt::Display;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SyntaxKind {
    /// `,` character.
    Comma,
    /// `(` character.
    LeftParen,
    /// `)` character.
    RightParen,
    /// Empty characters ignored by parser.
    Empty,
    /// Comment started by `//`.
    Comment,
    /// End of line.
    Eol,

    /// Unsigned integer.
    Integer,

    /// identifier started by a hash `#`.
    Ident,

    /// `define` statement.
    Define,
    /// `place` statement.
    Place,
    /// `push` statement.
    Push,
    /// `expect` statement.
    Expect,
    /// [parabox::ProtoType::Wall]
    Wall,
    /// [parabox::ProtoType::Box]
    Box,
    /// [parabox::ProtoType::Alias]
    Alias,
    /// [parabox::ProtoType::Infinity]
    Infinity,
    /// [parabox::ProtoType::Epsilon]
    Epsilon,
    /// [parabox::ProtoType::Void]
    Void,
    /// Property `size`.
    Size,
    /// Property `Ref`.
    Ref,
    /// Property `solid`.
    Solid,
    /// Keyword `at`.
    At,
    /// Keyword `in`.
    In,
    /// Keyword `orphan`.
    Orphan,
    /// Keyword `north`.
    North,
    /// Keyword `south`.
    South,
    /// Keyword `east`.
    East,
    /// Keyword `west`.
    West,
    /// Keyword `moved`.
    Moved,
    /// Keyword `static`.
    Static,
}

impl SyntaxKind {
    pub fn is_skipped(&self) -> bool {
        matches!(self, SyntaxKind::Empty | SyntaxKind::Comment)
    }

    pub fn is_eof(&self) -> bool {
        matches!(self, SyntaxKind::Eol)
    }

    pub fn is_proto(&self) -> bool {
        matches!(
            self,
            SyntaxKind::Wall
                | SyntaxKind::Box
                | SyntaxKind::Alias
                | SyntaxKind::Infinity
                | SyntaxKind::Epsilon
                | SyntaxKind::Void
        )
    }

    pub fn proto_needs_size(&self) -> bool {
        matches!(
            self,
            SyntaxKind::Box | SyntaxKind::Epsilon | SyntaxKind::Void
        )
    }

    pub fn proto_needs_reference(&self) -> bool {
        matches!(
            self,
            SyntaxKind::Alias | SyntaxKind::Infinity | SyntaxKind::Epsilon
        )
    }

    pub fn is_direction(&self) -> bool {
        matches!(
            self,
            SyntaxKind::North | SyntaxKind::South | SyntaxKind::East | SyntaxKind::West
        )
    }
}

impl Display for SyntaxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SyntaxKind::Comma => "`,`",
            SyntaxKind::LeftParen => "`(`",
            SyntaxKind::RightParen => "`)`",
            SyntaxKind::Comment => "comment",
            SyntaxKind::Empty => "empty character",
            SyntaxKind::Eol => "end of line",
            SyntaxKind::Integer => "integer",
            SyntaxKind::Ident => "identifier",
            SyntaxKind::Define => "`define`",
            SyntaxKind::Place => "`place`",
            SyntaxKind::Push => "`push`",
            SyntaxKind::Expect => "`expect`",
            SyntaxKind::Wall => "`wall`",
            SyntaxKind::Box => "`box`",
            SyntaxKind::Alias => "`alias`",
            SyntaxKind::Infinity => "`infinity`",
            SyntaxKind::Epsilon => "`epsilon`",
            SyntaxKind::Void => "`void`",
            SyntaxKind::Size => "`size`",
            SyntaxKind::Ref => "`ref`",
            SyntaxKind::Solid => "`solid`",
            SyntaxKind::At => "`at`",
            SyntaxKind::In => "`in`",
            SyntaxKind::Orphan => "`orphan`",
            SyntaxKind::North => "`north`",
            SyntaxKind::South => "`south`",
            SyntaxKind::East => "`east`",
            SyntaxKind::West => "`west`",
            SyntaxKind::Moved => "`moved`",
            SyntaxKind::Static => "`static`",
        };

        write!(f, "{}", str)
    }
}
