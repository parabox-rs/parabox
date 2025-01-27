//! The Parabox game engine.
//!
//! # Overview
//!
//! The game is about pushing blocks. Players can control one of the blocks and
//! move it to push others. The goal is to move the blocks to the target.
//!
//! The game world is completely made up of blocks. Some blocks have internal
//! structures, squares of  given sizes, and may contain other blocks. A block
//! may be contained in itself.
//!
//! In a game, each block is contained in a block, so the position of a block
//! can be described by its container and the coordinates inside the container.
//! Blocks that are not contained in any block are said to be _orphans_.
//!
//! The basic block types are _wall_ and _box_, where a wall has no internal
//! structures and cannot be pushed, while a box is the contrast.
//!
//! Each time a box is pushed, it will try to move to the target position. If
//! the position is taken by another box, it will be pushed together. If it is
//! blocked by a wall, then the pushing will fail, and nothing happens.
//!
//! Also, when a box is pushed out of the boundary, it will be pushed out of the
//! container and become a brother of the container (i.e., a child of the
//! container of its container). It is suddenly largened and comes out of the
//! container.
//!
//! Conversely, when a box pushes another box into a wall, the first box will
//! then try to enter the second box, becoming a child of it.
//!
//! Finally, if the entering also fails (because it might be blocked by
//! something inside the second box), the first box will try to eat the second
//! block, by letting the second box become a child of it.
//!
//! In conclusion, a block can push, exit, enter and eat. These are the four
//! basic movements in the game.
//!
//! # Usage
//!
//! - [`World`]: The game world that contains all the blocks. Use this struct to
//!   manage the blocks and perform game operations.
//! - [`ProtoType`]: The prototype of a block. Use this enum to create a block.
//! - [`BlockKey`]: The key of a block. Generated by the world when creating a
//!  block. Used to refer to the generated block.
//! - [`Position`]: The position of a block.
//! - [`Direction`]: The direction of a movement.
//!
//! [`World`]: crate::World
//! [`ProtoType`]: crate::ProtoType
//! [`BlockKey`]: crate::BlockKey
//! [`Position`]: crate::Position
//! [`Direction`]: crate::Direction
//!
//! # Example
//!
//! ```
//! # use parabox::{Direction, Position, ProtoType, World};
//!
//! let mut world = World::new();
//!
//! let container = world.insert(ProtoType::Box { size: (5, 5) });
//! let player = world.insert(ProtoType::Box { size: (1, 1) });
//! let block = world.insert(ProtoType::Box { size: (1, 1) });
//!
//! {
//!     // Make `player` solid by placing a wall in it
//!     let wall = world.insert(ProtoType::Wall);
//!     world.place(wall, Position::inside(player, (0, 0)));
//! }
//! {
//!     // Make `block` solid by placing a wall in it
//!     let wall = world.insert(ProtoType::Wall);
//!     world.place(wall, Position::inside(block, (0, 0)));
//! }
//!
//! world.place(player, Position::inside(container, (0, 2)));
//! world.place(block, Position::inside(container, (1, 2)));
//!
//! let result = world.push(player, Direction::East).expect("push failed");
//!
//! assert!(result, "the blocks remain static");
//! assert_eq!(world.position(player), Position::inside(container, (1, 2)));
//! assert_eq!(world.position(block), Position::inside(container, (2, 2)));
//! ```
//!
//! # Related crates
//!
//! - [`parabox-parser`]: Implements a script language for executing commands in
//!   the game.
//!
//! [`parabox-parser`]: https://crates.io/crates/parabox-parser

#![allow(dead_code)]
#![warn(missing_docs)]

extern crate self as parabox;

mod block;
mod world;

pub use block::{Block, BlockKey, Info, Position, ProtoType, Size, State};
pub use world::{Direction, MoveError, MoveResult, World};
