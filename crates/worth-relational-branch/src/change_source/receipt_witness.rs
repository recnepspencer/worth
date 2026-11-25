use worth_proof::{
    AssumptionBasis, AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness,
    CurrentValidity, ExecutionReadyRecipe, FreshnessScopedBasis, LoweredRecipeDxExt, Recipe,
    Resolved, ResolvedRecipeDxExt, Unresolved, UnresolvedRecipeDxExt,
};

use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{PartitionId, VersionId};

/// What a receipt was asked for: one commit, by one runtime, optionally
/// narrowed to one partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeReceiptRequest {
    pub runtime_instance_id: u64,
    pub commit_id: CommitId,
    pub partition_id: Option<PartitionId>,
}

/// What the retained commit resolved the request to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeReceiptBasis {
    pub version_id: VersionId,
    pub selected_branch_id: BranchId,
    pub authoring_branch_id: BranchId,
}

pub(super) type ChangeReceiptUnresolved = Recipe<Unresolved, ChangeReceiptRequest>;

type ChangeReceiptEvidence =
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<ChangeReceiptBasis>>;

pub(super) type ChangeReceiptResolved =
    Recipe<Resolved, ChangeReceiptRequest, ChangeReceiptEvidence>;

pub(super) type ChangeReceiptProof =
    ExecutionReadyRecipe<ChangeReceiptRequest, ChangeReceiptEvidence>;

struct ChangeReceiptResolutionAuthority {
    _private: (),
}

impl AuthorityMarker for ChangeReceiptResolutionAuthority {}

struct CanonicalChangeConsistencyCapability {
    _private: (),
}

impl CapabilityMarker for CanonicalChangeConsistencyCapability {}

struct ChangeReceiptReadinessAuthority {
    _private: (),
}

impl AuthorityMarker for ChangeReceiptReadinessAuthority {}

pub fn begin_change_receipt(request: ChangeReceiptRequest) -> ChangeReceiptUnresolved {
    Recipe::new(request)
}

pub fn resolve_change_receipt(
    unresolved: ChangeReceiptUnresolved,
    basis: ChangeReceiptBasis,
) -> ChangeReceiptResolved {
    unresolved.resolve_with(
        AuthorityWitness::from_authority_marker(ChangeReceiptResolutionAuthority { _private: () }),
        basis,
    )
}

/// Admit a resolved receipt once its change passed the consistency checks.
pub fn admit_consistent_change(resolved: ChangeReceiptResolved) -> ChangeReceiptProof {
    let runtime_instance_id = resolved.payload().runtime_instance_id;
    resolved
        .lower_with(CapabilityWitness::from_capability_marker(
            CanonicalChangeConsistencyCapability { _private: () },
        ))
        .ready_with(
            AuthorityWitness::from_authority_marker(ChangeReceiptReadinessAuthority {
                _private: (),
            }),
            runtime_instance_id,
        )
}
