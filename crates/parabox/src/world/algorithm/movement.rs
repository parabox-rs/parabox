use super::rational::Rational;
use crate::{BlockKey, Position, ProtoType, World};
use parabox_macros::trace_func;
use std::fmt::Debug;
use tracing::instrument;

/// The direction of a movement.
#[allow(missing_docs)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /// The corresponding vector of the direction.
    pub fn delta(self) -> (isize, isize) {
        match self {
            Direction::North => (0, 1),
            Direction::South => (0, -1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }

    /// The opposite direction.
    pub fn opposite(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

/// An arrow from the source.
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct SourceArrow {
    pub position: Position,
    pub direction: Direction,
    pub precise: Rational,
}

impl SourceArrow {
    pub fn new(position: Position, direction: Direction, precise: Rational) -> Self {
        Self {
            position,
            direction,
            precise,
        }
    }
}
impl Debug for SourceArrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Source({:?}@{:?}+{:?})",
            self.position, self.direction, self.precise
        )
    }
}

/// An arrow into the target.
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct TargetArrow {
    pub position: Position,
    pub direction: Direction,
    pub precise: Rational,
}

impl TargetArrow {
    pub fn new(position: Position, direction: Direction, precise: Rational) -> Self {
        Self {
            position,
            direction,
            precise,
        }
    }
}

impl Debug for TargetArrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Target({:?}@{:?}+{:?})",
            self.position, self.direction, self.precise
        )
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct ExitInfo {
    pub from: BlockKey,
    pub direction: Direction,
    pub precise: Rational,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct EnterInfo {
    pub into: BlockKey,
    pub direction: Direction,
    pub precise: Rational,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct EatInfo {
    pub eat: BlockKey,
    pub ate: BlockKey,
    pub direction: Direction,
}

#[derive(Copy, Clone)]
pub(crate) struct Movement {
    pub key: BlockKey,
    pub target: Position,
}

impl Movement {
    pub fn new(key: BlockKey, target: Position) -> Self {
        Self { key, target }
    }
}

impl Debug for Movement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Movement({:?} -> {:?})", self.key, self.target)
    }
}

/// The errors that may occur during a movement.
///
/// This is due to the lazy creation of [ProtoType::Infinity],
/// [ProtoType::Epsilon] and [ProtoType::Void] blocks. They are designed to be
/// automatically created when dealing with unresolved infinite recursion or
/// undefined exit.
///
/// When an exit occurs in an orphan, infinite exit or entering occurs on a
/// block without an infinity or epsilon reference, this error will be returned.
#[derive(Debug)]
pub enum MoveError {
    /// Trying to exit the orphan.
    Orphan(BlockKey),
    /// Trying to resolve the infinite exit from the block.
    NoInfinity(BlockKey),
    /// Trying to resolve the infinite entering to the block.
    NoEpsilon(BlockKey),
}

/// The result of a movement, just an alias of [Result] with [MoveError].
pub type MoveResult<T> = Result<T, MoveError>;

pub trait IntoMoveResult<T> {
    fn orphan(self, key: BlockKey) -> MoveResult<T>;

    fn no_infinity(self, key: BlockKey) -> MoveResult<T>;

    fn no_epsilon(self, key: BlockKey) -> MoveResult<T>;
}

impl<T> IntoMoveResult<T> for Option<T> {
    #[inline]
    fn orphan(self, key: BlockKey) -> MoveResult<T> {
        self.ok_or(MoveError::Orphan(key))
    }

    #[inline]
    fn no_infinity(self, key: BlockKey) -> MoveResult<T> {
        self.ok_or(MoveError::NoInfinity(key))
    }

    #[inline]
    fn no_epsilon(self, key: BlockKey) -> MoveResult<T> {
        self.ok_or(MoveError::NoEpsilon(key))
    }
}

pub struct MoveProcessor;

impl MoveProcessor {
    #[trace_func]
    #[instrument(skip(self, world))]
    pub fn exit(&self, world: &World, info: ExitInfo) -> MoveResult<SourceArrow> {
        let position = info.from.get(world).state.position;
        position.container.orphan(info.from)?;
        Ok(SourceArrow::new(position, info.direction, info.precise))
    }

    #[trace_func]
    #[instrument(skip(self, world))]
    pub fn infinity(&self, world: &World, info: ExitInfo) -> MoveResult<ExitInfo> {
        Ok(ExitInfo {
            from: info.from.get(world).info.infinity.no_infinity(info.from)?,
            direction: info.direction,
            precise: info.precise,
        })
    }

    #[trace_func]
    #[instrument(skip(self, world))]
    pub fn alias(&self, world: &World, mut info: EnterInfo) -> EnterInfo {
        loop {
            match info.into.get(world).proto {
                ProtoType::Alias { reference } => {
                    info = EnterInfo {
                        into: reference,
                        direction: info.direction,
                        precise: info.precise,
                    }
                }
                _ => break info,
            }
        }
    }

    #[trace_func]
    #[instrument(skip(self, world))]
    pub fn enter(&self, world: &World, info: EnterInfo) -> Option<TargetArrow> {
        let container = info.into.get(world);

        if container.proto.is_solid() {
            return None;
        }

        let tangent = match info.direction {
            Direction::North => 0,
            Direction::South => container.proto.height() - 1,
            Direction::East => 0,
            Direction::West => container.proto.width() - 1,
        };

        let normal = match info.direction {
            Direction::North | Direction::South => container.proto.width(),
            Direction::East | Direction::West => container.proto.height(),
        };

        let (offset, precise) = (info.precise * normal).split();

        let pos = match info.direction {
            Direction::North | Direction::South => (offset, tangent),
            Direction::East | Direction::West => (tangent, offset),
        };

        Some(TargetArrow::new(
            Position::inside(info.into, pos),
            info.direction,
            precise,
        ))
    }

    #[trace_func]
    #[instrument(skip(self, world))]
    pub fn epsilon(&self, world: &World, info: EnterInfo) -> MoveResult<EnterInfo> {
        Ok(EnterInfo {
            into: info.into.get(world).info.epsilon.no_epsilon(info.into)?,
            direction: info.direction,
            precise: info.precise,
        })
    }

    #[trace_func]
    #[instrument(skip(self, _world))]
    pub fn eat(&self, _world: &World, info: EatInfo) -> EnterInfo {
        EnterInfo {
            into: info.eat,
            direction: info.direction.opposite(),
            precise: Rational::HALF,
        }
    }
}
