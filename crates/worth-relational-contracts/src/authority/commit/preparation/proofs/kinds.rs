use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreparationProofKind {
    PartitionDisjoint,
    InvariantGroupDisjoint,
    FragmentIdentityDisjoint,
    ReadOnlyShared,
    RequiresSerial,
}
