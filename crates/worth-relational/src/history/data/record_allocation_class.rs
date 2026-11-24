use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// Separates entity and relation allocation domains within each partition.
pub enum RecordAllocationClass {
    Entity,
    Relation,
}
