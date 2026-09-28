use crate::facade::{BridgeAuthoritativePatchLoweringCounters, BridgeCommittedPatchEnvelope};

use super::change_publication::ChangeLoweringContext;
use worth_relational::facade::change_source::RelationalChangeReceipt;
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::PartitionId;

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
    error: crate::facade::BridgeRouteError,
    counters: BridgeAuthoritativePatchLoweringCounters,
}

impl RelationalBridgePublicationDenial {
    pub(super) const fn new(
        error: crate::facade::BridgeRouteError,
        counters: BridgeAuthoritativePatchLoweringCounters,
    ) -> Self {
        Self { error, counters }
    }

    pub fn error(&self) -> &crate::facade::BridgeRouteError {
        &self.error
    }

    pub fn kind(&self) -> crate::facade::BridgeRouteErrorKind {
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

/// A Bridge envelope lowered from one Relational change receipt, rather than
/// assembled from detached items.
///
/// Only receipt lowering mints it, so its provenance always names the
/// receipt's runtime, commit, and partition.
pub struct RelationalBridgePatchPublication {
    envelope: BridgeCommittedPatchEnvelope,
    runtime_instance_id: u64,
    commit_id: CommitId,
    relational_partition_id: Option<PartitionId>,
    graph_role: std::sync::Arc<str>,
    partition_role: Option<worth_foundational::facade::TruthPartitionRole>,
    adapter_semantic_identity: std::sync::Arc<str>,
    source_basis: std::sync::Arc<str>,
}

impl RelationalBridgePatchPublication {
    pub(super) fn mint(
        receipt: &RelationalChangeReceipt,
        envelope: BridgeCommittedPatchEnvelope,
        context: &ChangeLoweringContext<'_>,
        adapter_semantic_identity: std::sync::Arc<str>,
        source_basis: std::sync::Arc<str>,
    ) -> Self {
        Self {
            envelope,
            runtime_instance_id: receipt.runtime_instance_id(),
            commit_id: receipt.commit_id(),
            relational_partition_id: receipt.partition_id(),
            graph_role: context.graph_role.clone(),
            partition_role: context.partition_role.cloned(),
            adapter_semantic_identity,
            source_basis,
        }
    }

    pub fn bridge_envelope(&self) -> &BridgeCommittedPatchEnvelope {
        &self.envelope
    }

    pub fn lowering_counters(&self) -> &BridgeAuthoritativePatchLoweringCounters {
        self.envelope.patch_summary().authoritative_lowering()
    }

    pub fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    pub fn graph_role(&self) -> &str {
        &self.graph_role
    }

    pub fn adapter_semantic_identity(&self) -> &str {
        &self.adapter_semantic_identity
    }

    pub fn source_basis(&self) -> &str {
        &self.source_basis
    }

    pub fn partition_role(&self) -> Option<&worth_foundational::facade::TruthPartitionRole> {
        self.partition_role.as_ref()
    }

    pub fn relational_partition_id(&self) -> Option<PartitionId> {
        self.relational_partition_id
    }

    pub(crate) fn into_bridge_envelope(self) -> BridgeCommittedPatchEnvelope {
        self.envelope
    }
}
