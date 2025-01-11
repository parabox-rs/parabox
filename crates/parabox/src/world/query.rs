use super::world::World;
use crate::block::{Block, BlockKey, Position};

#[derive(Debug)]
pub(crate) enum PositionState {
    Void,
    Empty,
    OutofBound,
    Present(BlockKey),
}

impl PositionState {
    pub(crate) fn is_present(&self) -> bool {
        match self {
            PositionState::Present(_) => true,
            _ => false,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            PositionState::Empty => true,
            _ => false,
        }
    }

    pub(crate) fn as_option(&self) -> Option<BlockKey> {
        match self {
            PositionState::Present(block) => Some(*block),
            _ => None,
        }
    }
}

impl World {
    /// Gets the position of a block.
    pub fn position(&self, block: BlockKey) -> Position {
        self.blocks[block].state.position
    }

    pub(crate) fn position_state(&self, position: Position) -> PositionState {
        if let Some(container) = position.container {
            let container = &self.blocks[container];
            let (x, y) = position.pos;

            if container.proto.contains((x, y)) {
                if let Some(block) = container.state.interior[x][y] {
                    PositionState::Present(block)
                } else {
                    PositionState::Empty
                }
            } else {
                PositionState::OutofBound
            }
        } else {
            PositionState::Void
        }
    }
}

impl BlockKey {
    #[inline]
    pub(crate) fn get(self, world: &World) -> &Block {
        &world.blocks[self]
    }

    #[inline]
    pub(crate) fn get_mut(self, world: &mut World) -> &mut Block {
        &mut world.blocks[self]
    }
}
