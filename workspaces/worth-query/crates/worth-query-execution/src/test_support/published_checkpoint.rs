//! Exact published truth, separate from D7's burned allocator identities.
use worth_relational::facade::{
    branch::RelationalBranchBasisDescriptor,
    durability::{RelationalCheckpointMember, RelationalNativeCheckpoint},
    runtime::RelationalRuntime,
};

#[derive(Debug)]
pub(crate) struct PublishedCheckpoint {
    basis: RelationalBranchBasisDescriptor,
    native: RelationalNativeCheckpoint,
}

impl PartialEq for PublishedCheckpoint {
    fn eq(&self, other: &Self) -> bool {
        self.basis == other.basis
            && self.native.covered_position() == other.native.covered_position()
            && self.native.covered_branch_transitions() == other.native.covered_branch_transitions()
            && self.native.members() == other.native.members()
    }
}
impl Eq for PublishedCheckpoint {}

pub(crate) fn capture(runtime: &RelationalRuntime) -> PublishedCheckpoint {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let native = runtime
        .durability_authority()
        .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    // Retain the owner's capture, before transport decoding discards its
    // member handles. Compare every member's locator AND bytes, never an empty
    // transported member set or only its commit identity. D7's allocator
    // frontiers live in the manifest and may advance on a refused candidate.
    assert!(native.covered_position().is_some());
    assert!(native.covered_branch_transitions().is_some());
    assert!(!native.members().is_empty());
    assert!(native
        .members()
        .iter()
        .all(|member| matches!(member, RelationalCheckpointMember::New { .. })));
    PublishedCheckpoint {
        basis: basis.descriptor().clone(),
        native,
    }
}
