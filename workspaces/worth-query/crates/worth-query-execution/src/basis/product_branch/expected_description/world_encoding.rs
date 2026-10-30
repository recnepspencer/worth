//! Query's wire encoding of the World owner's descriptive coordinates.
use serde::{Deserialize, Serialize};
use worth_runtime_world::facade::ProductBranchReferenceDescription;

#[derive(Serialize, Deserialize)]
#[serde(remote = "ProductBranchReferenceDescription", deny_unknown_fields)]
pub(super) struct ReferenceDescription {
    pub format_version: u16,
    pub owner_ordinal: u64,
    pub branch_name: String,
    pub lifecycle_ordinal: u64,
    pub reference_generation: u64,
    pub composite_commit_ordinal: u64,
}
