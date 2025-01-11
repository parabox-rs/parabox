use super::types::Size;
use super::BlockKey;

/// The prototype of a block.
///
/// A prototype can be evaluted in the following 2 aspects:
/// - *static*: this block cannot be pushed
/// - *solid*: this block has no interior and cannot be entered
///
/// Hollow types all have an internal structure of a given size, which is
/// specified in the `size` field of the variant.
///
/// Some types may refer to another block, transferring exits or enterings to
/// that block. This is specified in the `reference` field of the variant.
///
/// The following prototypes are supported:
/// - [ProtoType::Wall]: a static, solid block
/// - [ProtoType::Box]: a hollow block
/// - [ProtoType::Alias]: a solid block that transfers enterings to another
///   block
/// - [ProtoType::Infinity]: a solid block that resolves infinite exits by
///   transfering them to another block
/// - [ProtoType::Epsilon]: a solid block that resolves infinite enterings by
///   transfering them to another block
/// - [ProtoType::Void]: a static, hollow block that only allow pushings for the
///   child blocks
#[allow(missing_docs)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProtoType {
    Wall,
    Box { size: Size },
    Alias { reference: BlockKey },
    Infinity { reference: BlockKey },
    Epsilon { size: Size, reference: BlockKey },
    Void { size: Size },
}

impl ProtoType {
    /// Returns a [ProtoType::Wall].
    pub fn wall() -> Self {
        ProtoType::Wall
    }

    /// Returns a [ProtoType::Box].
    pub fn box_(size: Size) -> Self {
        ProtoType::Box { size }
    }

    /// Returns a [ProtoType::Alias].
    pub fn alias(reference: BlockKey) -> Self {
        ProtoType::Alias { reference }
    }

    /// Returns a [ProtoType::Infinity].
    pub fn infinity(reference: BlockKey) -> Self {
        ProtoType::Infinity { reference }
    }

    /// Returns a [ProtoType::Epsilon].
    pub fn epsilon(size: Size, reference: BlockKey) -> Self {
        ProtoType::Epsilon { size, reference }
    }

    /// Returns a [ProtoType::Void].
    pub fn void(size: Size) -> Self {
        ProtoType::Void { size }
    }
}

impl ProtoType {
    /// Returns the internal size of the block.
    pub fn size(self) -> Size {
        match self {
            ProtoType::Box { size } => size,
            ProtoType::Epsilon { size, .. } => size,
            ProtoType::Void { size } => size,
            _ => Size::default(),
        }
    }

    /// Returns the internal height of the block.
    pub fn height(self) -> usize {
        self.size().1
    }

    /// Returns the internal width of the block.
    pub fn width(self) -> usize {
        self.size().0
    }

    /// Returns whether the block is solid.
    ///
    /// See [ProtoType] for more information.
    pub fn is_solid(&self) -> bool {
        match self {
            ProtoType::Wall => true,
            ProtoType::Alias { .. } => true,
            ProtoType::Infinity { .. } => true,
            _ => false,
        }
    }

    /// Returns whether the block is hollow.
    ///
    /// See [ProtoType] for more information.
    pub fn is_hollow(&self) -> bool {
        match self {
            ProtoType::Box { .. } => true,
            ProtoType::Epsilon { .. } => true,
            ProtoType::Void { .. } => true,
            _ => false,
        }
    }

    /// Returns whether the block is static.
    pub fn is_static(&self) -> bool {
        matches!(self, ProtoType::Wall | ProtoType::Void { .. })
    }

    /// Returns whether the block is void.
    pub fn is_void(&self) -> bool {
        matches!(self, ProtoType::Void { .. })
    }

    /// Returns the optional reference.
    pub fn reference(&self) -> Option<BlockKey> {
        match self {
            ProtoType::Alias { reference: bind } => Some(*bind),
            ProtoType::Infinity { reference: bind } => Some(*bind),
            ProtoType::Epsilon {
                reference: bind, ..
            } => Some(*bind),
            _ => None,
        }
    }

    /// Returns whether the block can be aliased.
    pub fn can_alias(&self) -> bool {
        match self {
            ProtoType::Wall => false,
            ProtoType::Box { .. } => true,
            ProtoType::Alias { .. } => false,
            ProtoType::Infinity { .. } => false,
            ProtoType::Epsilon { .. } => true,
            ProtoType::Void { .. } => false,
        }
    }

    /// Returns whether the block can be referred to by infinity.
    pub fn can_infinity(&self) -> bool {
        match self {
            ProtoType::Wall => false,
            ProtoType::Box { .. } => true,
            ProtoType::Alias { .. } => false,
            ProtoType::Infinity { .. } => true,
            ProtoType::Epsilon { .. } => true,
            ProtoType::Void { .. } => false,
        }
    }

    /// Returns whether the block can be referred to by epsilon.
    pub fn can_epsilon(&self) -> bool {
        match self {
            ProtoType::Wall => false,
            ProtoType::Box { .. } => true,
            ProtoType::Alias { .. } => false,
            ProtoType::Infinity { .. } => false,
            ProtoType::Epsilon { .. } => true,
            ProtoType::Void { .. } => false,
        }
    }
}

impl ProtoType {
    pub(crate) fn contains(&self, (x, y): Size) -> bool {
        let (width, height) = self.size();
        x < width && y < height
    }
}
