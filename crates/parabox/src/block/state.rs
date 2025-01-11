use super::types::Size;
use super::{BlockKey, Position};

/// The state of a block.
pub struct State {
    /// The position of the block.
    pub position: Position,
    /// The interior matrix of the block.
    ///
    /// The matrix is represented as a 2D vector, where the first dimension
    /// represents the x-axis and the second dimension represents the y-axis.
    ///
    /// The matrix is filled with `Option<BlockKey>`, where `None` represents
    /// an empty space and `Some(BlockKey)` represents a block.
    pub interior: Vec<Vec<Option<BlockKey>>>,
}

impl State {
    pub(crate) fn new(size: Size) -> Self {
        let (width, height) = size;
        let position = Position::default();
        let interior = vec![vec![None; height]; width];

        Self { position, interior }
    }
}
