use serde::{Deserialize, Serialize};

/// Subject class governed by a persistent name scheme in Relational.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PersistentNameSubjectKind {
    Entity,
    Relation,
    Group,
}

impl PersistentNameSubjectKind {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Relation => "relation",
            Self::Group => "group",
        }
    }
}
