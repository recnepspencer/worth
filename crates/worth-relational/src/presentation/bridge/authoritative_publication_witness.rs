use std::sync::Arc;

use worth_proof::{
    AssumptionBasis, AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness,
    CurrentValidity, ExecutionReadyRecipe, FreshnessScopedBasis, LoweredRecipeDxExt, Recipe,
    Resolved, ResolvedRecipeDxExt, Unresolved, UnresolvedRecipeDxExt,
};

use crate::history::data::CommitId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RelationalPublicationRequest {
    pub(super) runtime_instance_id: u64,
    pub(super) commit_id: CommitId,
    pub(super) graph_role: Arc<str>,
    pub(super) relational_partition_id: Option<crate::identity::data::PartitionId>,
    pub(super) partition_role: Option<worth_foundational::facade::TruthPartitionRole>,
    pub(super) widening: Option<worth_runtime_bridge::facade::BridgeAspectChangeWideningCause>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RelationalPublicationBasis {
    pub(super) version_id: u64,
    pub(super) branch_id: Arc<str>,
    pub(super) adapter_semantic_identity: Arc<str>,
    pub(super) source_basis: Arc<str>,
}

pub(super) type PublicationUnresolvedRecipe = Recipe<Unresolved, RelationalPublicationRequest>;

type PublicationResolvedRecipe = Recipe<
    Resolved,
    RelationalPublicationRequest,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<RelationalPublicationBasis>>,
>;

pub(super) type PublicationReadyRecipe = ExecutionReadyRecipe<
    RelationalPublicationRequest,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<RelationalPublicationBasis>>,
>;

struct RelationalPublicationResolutionAuthority {
    _private: (),
}

impl AuthorityMarker for RelationalPublicationResolutionAuthority {}

struct CanonicalPatchLoweringCapability {
    _private: (),
}

impl CapabilityMarker for CanonicalPatchLoweringCapability {}

struct RelationalPublicationReadinessAuthority {
    _private: (),
}

impl AuthorityMarker for RelationalPublicationReadinessAuthority {}

pub(super) fn begin_publication(
    request: RelationalPublicationRequest,
) -> PublicationUnresolvedRecipe {
    Recipe::new(request)
}

pub(super) fn resolve_publication(
    unresolved: Recipe<Unresolved, RelationalPublicationRequest>,
    basis: RelationalPublicationBasis,
) -> PublicationResolvedRecipe {
    unresolved.resolve_with(
        AuthorityWitness::from_authority_marker(RelationalPublicationResolutionAuthority {
            _private: (),
        }),
        basis,
    )
}

pub(super) fn admit_lowered_publication(
    resolved: PublicationResolvedRecipe,
) -> PublicationReadyRecipe {
    let runtime_instance_id = resolved.payload().runtime_instance_id;
    resolved
        .lower_with(CapabilityWitness::from_capability_marker(
            CanonicalPatchLoweringCapability { _private: () },
        ))
        .ready_with(
            AuthorityWitness::from_authority_marker(RelationalPublicationReadinessAuthority {
                _private: (),
            }),
            runtime_instance_id,
        )
}
