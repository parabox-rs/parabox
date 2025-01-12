use crate::{Block, BlockKey, Position, ProtoType};
use slotmap::SlotMap;
use std::ops::Index;

pub type Blocks = SlotMap<BlockKey, Block>;

/// Controls the game world and implements the game logic.
pub struct World {
    pub(crate) blocks: Blocks,
}

impl World {
    /// Creates a new world.
    pub fn new() -> Self {
        Self {
            blocks: SlotMap::with_key(),
        }
    }

    /// Inserts a block with the given prototype into the world, returning the
    /// key to the block.
    pub fn insert(&mut self, proto: ProtoType) -> BlockKey {
        // Insert the block and get the key.
        let key = self.blocks.insert_with_key(|key| Block::new(key, proto));

        // Update reference relationships.
        match proto {
            ProtoType::Alias { reference, .. } => {
                self.blocks[reference].info.references.insert(key);
            }
            ProtoType::Infinity { reference, .. } => {
                self.blocks[reference].info.infinity = Some(key);
            }
            ProtoType::Epsilon { reference, .. } => {
                self.blocks[reference].info.epsilon = Some(key);
            }
            _ => {}
        };

        key
    }

    /// Removes the block from the world.
    pub fn remove(&mut self, key: BlockKey) {
        // Remove the block from its container.
        if let Some(container) = self.blocks[key].state.position.container {
            let (x, y) = self.blocks[key].state.position.pos;
            let interior = &mut self.blocks[container].state.interior;
            interior[x][y] = None;
        }

        // Remove the block
        let block = self.blocks.remove(key).unwrap();

        // Remove the children
        for row in block.state.interior {
            for cell in row {
                if let Some(child) = cell {
                    self.place(child, Position::default());
                }
            }
        }

        // Remove the reference owners.
        for alias in block.info.references {
            self.remove(alias);
        }
        if let Some(infinity) = block.info.infinity {
            self.remove(infinity);
        }
        if let Some(epsilon) = block.info.epsilon {
            self.remove(epsilon);
        }
    }

    /// Places the block at the given position.
    pub fn place(&mut self, key: BlockKey, position: Position) {
        // Remove the block from its current position.
        if let Some(current_container) = self.blocks[key].state.position.container {
            let (x, y) = self.blocks[key].state.position.pos;
            let interior = &mut self.blocks[current_container].state.interior;
            interior[x][y] = None;
        }

        // Place the block in the new position.
        if let Some(target_container) = position.container {
            let (x, y) = position.pos;
            let interior = &mut self.blocks[target_container].state.interior;
            interior[x][y] = Some(key);
        }

        // Set the position.
        self.blocks[key].state.position = position;
    }

    /// Returns a reference to the blocks in the world.
    ///
    /// The blocks are stored by [slotmap::SlotMap].
    pub fn blocks(&self) -> &Blocks {
        &self.blocks
    }
}

impl Index<BlockKey> for World {
    type Output = Block;

    fn index(&self, key: BlockKey) -> &Self::Output {
        &self.blocks[key]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut world = World::new();
        let key = world.insert(ProtoType::Wall);

        assert_eq!(world.blocks.len(), 1);
        assert_eq!(world.blocks[key].proto, ProtoType::Wall);
    }

    #[test]
    fn test_remove() {
        let mut world = World::new();
        let key = world.insert(ProtoType::Wall);
        world.remove(key);

        assert_eq!(world.blocks.len(), 0);
    }

    #[test]
    fn test_place() {
        let mut world = World::new();

        let container1 = world.insert(ProtoType::Box { size: (5, 5) });
        let container2 = world.insert(ProtoType::Box { size: (5, 5) });
        let block = world.insert(ProtoType::Box { size: (5, 5) });

        world.place(block, Position::inside(container1, (2, 1)));
        assert_eq!(world.blocks[container1].state.interior[2][1], Some(block));
        assert_eq!(
            world[block].state.position,
            Position::inside(container1, (2, 1))
        );

        world.place(block, Position::inside(container2, (3, 3)));
        assert_eq!(world.blocks[container1].state.interior[2][1], None);
        assert_eq!(world.blocks[container2].state.interior[3][3], Some(block));
        assert_eq!(
            world[block].state.position,
            Position::inside(container2, (3, 3))
        );

        world.place(block, Position::inside(block, (1, 1)));
        assert_eq!(world.blocks[container2].state.interior[3][3], None);
        assert_eq!(world.blocks[block].state.interior[1][1], Some(block));
        assert_eq!(world[block].state.position, Position::inside(block, (1, 1)));
    }
}
