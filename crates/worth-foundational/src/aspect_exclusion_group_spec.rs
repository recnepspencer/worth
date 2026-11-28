use serde::{Deserialize, Serialize};

/// Mutually exclusive aspects and whether the group requires one active member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectExclusionGroupSpec<Members = Vec<String>> {
    pub group_id: String,
    pub aspects: Members,
    pub required: bool,
}
