use super::BlockKey;
use std::collections::HashSet;

/// Information about a block.
///
/// The values won't change during the movements.
#[derive(Default)]
pub struct Info {
    /// The alias blocks referring to this block.
    pub references: HashSet<BlockKey>,
    /// The infinity block referring to this block.
    pub infinity: Option<BlockKey>,
    /// The epsilon block referring to this block.
    pub epsilon: Option<BlockKey>,
}
