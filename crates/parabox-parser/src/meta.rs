use crate::MetaProtoType;
use ecow::EcoString;
use parabox::{BlockKey, Position, ProtoType};
use parabox_parser::MetaPosition;
use std::collections::HashMap;
use std::fmt::Debug;

/// A table that maps block names to block keys and vice versa.
pub struct MetaTable {
    name_to_key: HashMap<EcoString, BlockKey>,
    key_to_name: HashMap<BlockKey, EcoString>,
}

impl MetaTable {
    /// Creates a new empty meta table.
    pub fn new() -> Self {
        Self {
            name_to_key: HashMap::new(),
            key_to_name: HashMap::new(),
        }
    }

    /// Removes a block by its name. Returns the key of the removed block.
    pub fn remove_by_name(&mut self, name: &EcoString) -> Option<BlockKey> {
        if let Some(key) = self.name_to_key.remove(name) {
            self.key_to_name.remove(&key);
            Some(key)
        } else {
            None
        }
    }

    /// Removes a block by its key. Returns the name of the removed block.
    pub fn remove_by_key(&mut self, key: &BlockKey) -> Option<EcoString> {
        if let Some(name) = self.key_to_name.remove(key) {
            self.name_to_key.remove(&name);
            Some(name)
        } else {
            None
        }
    }

    /// Inserts a block into the table. Returns the old key if the name already
    /// exists. Returns the old name is the key already exists.
    pub fn insert(
        &mut self,
        name: EcoString,
        key: BlockKey,
    ) -> (Option<BlockKey>, Option<EcoString>) {
        let old_key = self.name_to_key.insert(name.clone(), key).map(|old_key| {
            self.key_to_name.remove(&old_key);
            old_key
        });

        let old_name = self.key_to_name.insert(key, name).map(|old_name| {
            self.name_to_key.remove(&old_name);
            old_name
        });

        (old_key, old_name)
    }

    /// Checks if the table contains a block with the given name.
    pub fn contains_name(&self, name: &EcoString) -> bool {
        self.name_to_key.contains_key(name)
    }

    /// Checks if the table contains a block with the given key.
    pub fn contains_key(&self, key: &BlockKey) -> bool {
        self.key_to_name.contains_key(key)
    }

    /// Gets the key of a block by its name.
    pub fn get_key(&self, name: &EcoString) -> Option<BlockKey> {
        self.name_to_key.get(name).copied()
    }

    /// Gets the name of a block by its key.
    pub fn get_name(&self, key: &BlockKey) -> Option<EcoString> {
        self.key_to_name.get(key).cloned()
    }

    /// Gets the list of block names in the table. Sorted by name.
    pub fn names(&self) -> Vec<EcoString> {
        let mut names = self.name_to_key.keys().cloned().collect::<Vec<_>>();
        names.sort();
        names
    }

    /// Gets an iterator over the table. Sorted by name.
    pub fn iter(&self) -> impl Iterator<Item=(EcoString, BlockKey)> + '_ {
        self.names().into_iter().map(move |name| {
            let key = self.name_to_key.get(&name).unwrap();
            (name, *key)
        })
    }

    /// Converts the key meta to name meta.
    pub fn key_to_name<T: MetaKey>(&self, from: &T) -> Result<T::Target, BlockKey> {
        from.convert(self)
    }

    /// Converts the name meta to key meta.
    pub fn name_to_key<T: MetaName>(&self, from: &T) -> Result<T::Target, EcoString> {
        from.convert(self)
    }
}

impl Debug for MetaTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MetaTable {{\n")?;
        for (key, name) in self.key_to_name.iter() {
            write!(f, "  {key:?} => #{name}\n")?;
        }
        write!(f, "}}")
    }
}

/// A key meta that can be converted to the corresponding name meta.
///
/// There are structures that need to refer to other blocks. The data generated
/// by a code often uses the block name, while the data needed by the game world
/// uses the block key. This trait is used to convert between the two.
///
/// See also trait [MetaName].
pub trait MetaKey {
    /// The corresponding name meta type.
    type Target: MetaName<Target=Self>;

    /// Converts the key meta to the corresponding name meta.
    fn convert(&self, meta: &MetaTable) -> Result<Self::Target, BlockKey>;
}

/// A name meta that can be converted to the corresponding key meta.
///
/// There are structures that need to refer to other blocks. The data generated
/// by a code often uses the block name, while the data needed by the game world
/// uses the block key. This trait is used to convert between the two.
///
/// See also trait [MetaKey].
pub trait MetaName {
    /// The corresponding key meta type.
    type Target: MetaKey<Target=Self>;

    /// Converts the name meta to the corresponding key meta.
    fn convert(&self, meta: &MetaTable) -> Result<Self::Target, EcoString>;
}

impl<T: MetaKey> MetaKey for Option<T> {
    type Target = Option<T::Target>;

    fn convert(&self, meta: &MetaTable) -> Result<Self::Target, BlockKey> {
        match self {
            Some(key) => key.convert(meta).map(Some),
            None => Ok(None),
        }
    }
}

impl<T: MetaName> MetaName for Option<T> {
    type Target = Option<T::Target>;

    fn convert(&self, meta: &MetaTable) -> Result<Self::Target, EcoString> {
        match self {
            Some(name) => name.convert(meta).map(Some),
            None => Ok(None),
        }
    }
}

impl<T: MetaKey, E: Clone> MetaKey for Result<T, E> {
    type Target = Result<T::Target, E>;

    fn convert(&self, meta: &MetaTable) -> Result<Self::Target, BlockKey> {
        match self {
            Ok(key) => key.convert(meta).map(Ok),
            Err(e) => Ok(Err(e.clone())),
        }
    }
}

impl<T: MetaName, E: Clone> MetaName for Result<T, E> {
    type Target = Result<T::Target, E>;

    fn convert(&self, meta: &MetaTable) -> Result<Self::Target, EcoString> {
        match self {
            Ok(name) => name.convert(meta).map(Ok),
            Err(e) => Ok(Err(e.clone())),
        }
    }
}

impl MetaKey for BlockKey {
    type Target = EcoString;

    fn convert(&self, meta: &MetaTable) -> Result<Self::Target, BlockKey> {
        meta.get_name(self).ok_or(*self)
    }
}

impl MetaName for EcoString {
    type Target = BlockKey;

    fn convert(&self, meta: &MetaTable) -> Result<Self::Target, EcoString> {
        meta.get_key(self).ok_or_else(|| self.clone())
    }
}

impl MetaKey for Position {
    type Target = MetaPosition;

    fn convert(&self, meta: &MetaTable) -> Result<Self::Target, BlockKey> {
        Ok(MetaPosition {
            container: self.container.convert(meta)?,
            pos: self.pos,
        })
    }
}

impl MetaName for MetaPosition {
    type Target = Position;

    fn convert(&self, meta: &MetaTable) -> Result<Self::Target, EcoString> {
        Ok(Position {
            container: self.container.convert(meta)?,
            pos: self.pos,
        })
    }
}

impl MetaKey for ProtoType {
    type Target = MetaProtoType;

    fn convert(&self, meta: &MetaTable) -> Result<Self::Target, BlockKey> {
        match self {
            ProtoType::Wall => Ok(MetaProtoType::Wall),
            ProtoType::Box { size } => Ok(MetaProtoType::Box { size: *size }),
            ProtoType::Alias { reference } => Ok(MetaProtoType::Alias {
                reference: reference.convert(meta)?,
            }),
            ProtoType::Infinity { reference } => Ok(MetaProtoType::Infinity {
                reference: reference.convert(meta)?,
            }),
            ProtoType::Epsilon { reference, size } => Ok(MetaProtoType::Epsilon {
                reference: reference.convert(meta)?,
                size: *size,
            }),
            ProtoType::Void { size } => Ok(MetaProtoType::Void { size: *size }),
        }
    }
}

impl MetaName for MetaProtoType {
    type Target = ProtoType;

    fn convert(&self, meta: &MetaTable) -> Result<Self::Target, EcoString> {
        match self {
            MetaProtoType::Wall => Ok(ProtoType::Wall),
            MetaProtoType::Box { size } => Ok(ProtoType::Box { size: *size }),
            MetaProtoType::Alias { reference } => Ok(ProtoType::Alias {
                reference: reference.convert(meta)?,
            }),
            MetaProtoType::Infinity { reference } => Ok(ProtoType::Infinity {
                reference: reference.convert(meta)?,
            }),
            MetaProtoType::Epsilon { reference, size } => Ok(ProtoType::Epsilon {
                reference: reference.convert(meta)?,
                size: *size,
            }),
            MetaProtoType::Void { size } => Ok(ProtoType::Void { size: *size }),
        }
    }
}
