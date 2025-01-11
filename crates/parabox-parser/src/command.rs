use ecow::EcoString;
use parabox::{Direction, Size};
use std::fmt::{Debug, Display};

/// The key meta version of prototype.
///
/// See [parabox::ProtoType] for more information.
#[allow(missing_docs)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum MetaProtoType {
    Wall,
    Box { size: Size },
    Alias { reference: EcoString },
    Infinity { reference: EcoString },
    Epsilon { reference: EcoString, size: Size },
    Void { size: Size },
}

impl MetaProtoType {
    /// Returns the reference of the prototype if it has one.
    pub fn reference(&self) -> Option<EcoString> {
        match self {
            MetaProtoType::Alias { reference } => Some(reference.clone()),
            MetaProtoType::Infinity { reference } => Some(reference.clone()),
            MetaProtoType::Epsilon { reference, .. } => Some(reference.clone()),
            _ => None,
        }
    }

    /// Returns the size of the prototype if it has one.
    pub fn size(&self) -> Size {
        match self {
            MetaProtoType::Box { size } => *size,
            MetaProtoType::Epsilon { size, .. } => *size,
            MetaProtoType::Void { size } => *size,
            _ => Size::default(),
        }
    }

    /// Returns whether the prototype can be aliased.
    pub fn can_alias(&self) -> bool {
        match self {
            MetaProtoType::Wall => false,
            MetaProtoType::Box { .. } => true,
            MetaProtoType::Alias { .. } => false,
            MetaProtoType::Infinity { .. } => false,
            MetaProtoType::Epsilon { .. } => true,
            MetaProtoType::Void { .. } => false,
        }
    }

    /// Returns whether the prototype can be referred to by infinity.
    pub fn can_infinity(&self) -> bool {
        match self {
            MetaProtoType::Wall => false,
            MetaProtoType::Box { .. } => true,
            MetaProtoType::Alias { .. } => false,
            MetaProtoType::Infinity { .. } => true,
            MetaProtoType::Epsilon { .. } => true,
            MetaProtoType::Void { .. } => false,
        }
    }

    /// Returns whether the prototype can be referred to by epsilon.
    pub fn can_epsilon(&self) -> bool {
        match self {
            MetaProtoType::Wall => false,
            MetaProtoType::Box { .. } => true,
            MetaProtoType::Alias { .. } => false,
            MetaProtoType::Infinity { .. } => false,
            MetaProtoType::Epsilon { .. } => true,
            MetaProtoType::Void { .. } => false,
        }
    }
}

/// The key meta version of position.
///
/// See [parabox::Position] for more information.
#[allow(missing_docs)]
#[derive(Clone, Eq, PartialEq)]
pub struct MetaPosition {
    pub container: Option<EcoString>,
    pub pos: Size,
}

impl MetaPosition {
    /// Creates a new meta position.
    pub fn new(container: Option<EcoString>, pos: Size) -> Self {
        Self { container, pos }
    }

    /// Creates a new meta position inside a container.
    pub fn inside(container: EcoString, pos: Size) -> Self {
        Self::new(Some(container), pos)
    }

    /// Creates a new orphan meta position.
    pub fn orphan(pos: Size) -> Self {
        Self::new(None, pos)
    }
}

/// The assertion of a push command.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Assertion {
    /// No assertion.
    None,
    /// Asserting that the push is successful.
    Moved,
    /// Asserting that the push is blocked.
    Static,
}

/// The operation of a command.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Operation {
    /// Defines a new block with the prototype.
    Define(MetaProtoType),
    /// Places a block at the position.
    Place(MetaPosition),
    /// Pushes a block in a direction with an assertion.
    Push(Direction, Assertion),
    /// Expects a block at the position.
    Expect(MetaPosition),
}

/// A command to execute.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Command {
    /// The execution target.
    pub block: EcoString,
    /// The operation to execute.
    pub operation: Operation,
}

impl Command {
    /// Creates a new command.
    pub fn new(block: EcoString, operation: Operation) -> Self {
        Self { block, operation }
    }

    /// Creates a new define command.
    pub fn define(block: EcoString, proto: MetaProtoType) -> Self {
        Self::new(block, Operation::Define(proto))
    }

    /// Creates a new place command.
    pub fn place(block: EcoString, container: Option<EcoString>, pos: Size) -> Self {
        Self::new(block, Operation::Place(MetaPosition::new(container, pos)))
    }

    /// Creates a new push command.
    pub fn push(block: EcoString, direction: Direction, assertion: Assertion) -> Self {
        Self::new(block, Operation::Push(direction, assertion))
    }

    /// Creates a new expect command.
    pub fn expect(block: EcoString, container: Option<EcoString>, pos: Size) -> Self {
        Self::new(block, Operation::Expect(MetaPosition::new(container, pos)))
    }
}

impl Display for MetaPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(container) = &self.container {
            write!(f, "at {:?} in #{}", self.pos, container)
        } else {
            write!(f, "orphan")
        }
    }
}

impl Debug for MetaPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
