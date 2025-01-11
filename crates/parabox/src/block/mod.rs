mod info;
mod proto;
mod state;
mod types;

pub use info::Info;
pub use proto::ProtoType;
pub use state::State;
pub use types::{BlockKey, Position, Size};

/// A block in the world.
pub struct Block {
    /// The key of the block.
    pub key: BlockKey,
    /// The prototype of the block.
    pub proto: ProtoType,
    /// The state of the block.
    pub state: State,
    /// The information of the block.
    pub info: Info,
}

impl Block {
    pub(crate) fn new(key: BlockKey, proto: ProtoType) -> Self {
        Self {
            key,
            proto,
            state: State::new(proto.size()),
            info: Info::default(),
        }
    }
}
