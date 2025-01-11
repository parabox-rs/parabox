mod algorithm;
mod cycle;
mod movement;
mod rational;

use crate::{BlockKey, World};
use algorithm::Algorithm;

pub use movement::{Direction, MoveError, MoveResult};

impl World {
    /// Push a block in a direction.
    ///
    /// Returns:
    /// - `Ok(true)` if some movement occurs in the world.
    /// - `Ok(false)` if no movement occurs in the world.
    /// - `Err(MoveError)` if there is an error. See [MoveError].
    pub fn push(&mut self, key: BlockKey, direction: Direction) -> MoveResult<bool> {
        let mut algorithm = Algorithm::new();
        let result = algorithm.push(self, key, direction)?;

        if result {
            algorithm.commit(self);
        }

        Ok(result)
    }
}
