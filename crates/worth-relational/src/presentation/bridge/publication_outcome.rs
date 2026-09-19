use worth_runtime_bridge::facade::{
    BridgeAuthoritativePatchLoweringCounters, BridgeCommittedPatchEnvelope,
};

use super::authoritative_publication_witness::PublicationReadyRecipe;
use crate::history::data::CommitId;

pub type RelationalBridgePublicationOutcome = worth_proof::TransitionOutcome<
    RelationalBridgePatchPublication,
    RelationalBridgePublicationDenial,
    RelationalBridgePublicationDeferred,
    RelationalBridgePublicationStale,
    RelationalBridgePublicationRebindRequired,
    RelationalBridgePublicationFailure,
>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalBridgePublicationDenial {
    error: worth_runtime_bridge::facade::BridgeRouteError,
    counters: BridgeAuthoritativePatchLoweringCounters,
}

impl RelationalBridgePublicationDenial {
    pub(super) const fn new(
        error: worth_runtime_bridge::facade::BridgeRouteError,
        counters: BridgeAuthoritativePatchLoweringCounters,
    ) -> Self {
        Self { error, counters }
    }

    pub fn error(&self) -> &worth_runtime_bridge::facade::BridgeRouteError {
        &self.error
    }

    pub fn kind(&self) -> worth_runtime_bridge::facade::BridgeRouteErrorKind {
        self.error.kind()
    }

    pub const fn counters(&self) -> BridgeAuthoritativePatchLoweringCounters {
        self.counters
    }
}

impl std::fmt::Display for RelationalBridgePublicationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(formatter)
    }
}

impl std::error::Error for RelationalBridgePublicationDenial {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalBridgePublicationDeferred {
    CommitVisibilityPending,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalBridgePublicationStale {
    RuntimeAuthority,
    CommitNotRetained,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalBridgePublicationRebindRequired {
    GraphRole,
}

pub type RelationalBridgePublicationFailure = std::convert::Infallible;

/// Relational-owned proof that a Bridge envelope was derived from one admitted
/// canonical authoritative patch, rather than assembled from detached items.
pub struct RelationalBridgePatchPublication {
    envelope: BridgeCommittedPatchEnvelope,
    proof: PublicationReadyRecipe,
    _commit_identity: crate::identity_authority::RelationalSourceTruthAuthorityIdentity<
        u64,
        crate::identity_authority::RelationalCommitIdentityKind,
    >,
}

impl RelationalBridgePatchPublication {
    pub(super) fn mint(
        proof: PublicationReadyRecipe,
        envelope: BridgeCommittedPatchEnvelope,
    ) -> Self {
        let commit_identity = worth_foundational::facade::admit_foundational_authority_identity(
            proof.payload().commit_id.0,
            crate::identity_authority::relational_source_truth_authority(),
        );
        Self {
            envelope,
            proof,
            _commit_identity: commit_identity,
        }
    }

    pub fn bridge_envelope(&self) -> &BridgeCommittedPatchEnvelope {
        &self.envelope
    }

    pub fn lowering_counters(&self) -> &BridgeAuthoritativePatchLoweringCounters {
        self.envelope.patch_summary().authoritative_lowering()
    }

    pub fn runtime_instance_id(&self) -> u64 {
        self.proof.payload().runtime_instance_id
    }

    pub fn commit_id(&self) -> CommitId {
        self.proof.payload().commit_id
    }

    pub fn graph_role(&self) -> &str {
        &self.proof.payload().graph_role
    }

    pub fn adapter_semantic_identity(&self) -> &str {
        &self.proof.strong_basis().value().adapter_semantic_identity
    }

    pub fn source_basis(&self) -> &str {
        &self.proof.strong_basis().value().source_basis
    }

    pub fn partition_role(&self) -> Option<&worth_foundational::facade::TruthPartitionRole> {
        self.proof.payload().partition_role.as_ref()
    }

    pub fn relational_partition_id(&self) -> Option<crate::identity::data::PartitionId> {
        self.proof.payload().relational_partition_id
    }

    pub(crate) fn into_bridge_envelope(self) -> BridgeCommittedPatchEnvelope {
        self.envelope
    }
}
