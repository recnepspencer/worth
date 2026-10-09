//! Product-local semantic output correspondence owned by Query publication.

mod current_output;
pub(in crate::domain_computation::primary_graph) use current_output::RetainedOutputCurrentnessRead;
mod denial;
mod family_selection;
pub(in crate::domain_computation::primary_graph) use family_selection::{
    CheckpointPriorSelectionDenial, NativePriorCheckpointOutput,
};
mod input_cutoff;
mod input_reuse_key;
pub(in crate::domain_computation::primary_graph) mod invalidation;
mod native_output_witness;
mod native_prior_checkpoint;
mod partition_index;
mod performed_publication;
mod prepared_slot;
mod qualification;
mod recorded_output;
mod recorded_source_identity;
mod required_settlement;
pub(in crate::domain_computation::primary_graph) use required_settlement::{
    AcceptedCurrentCandidate, BoundCurrentAcceptedOutput, CurrentAcceptedResult,
    CurrentAcceptedStop,
};
mod resolution;
mod resources;
mod restoration;
mod retained_capacity;
mod retention;
mod settlement_identity;
pub(in crate::domain_computation) use invalidation::InvalidationEditAdmission;
pub(in crate::domain_computation) use invalidation::SourceInvalidationOwner;
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) mod registry_fixture;
#[cfg(test)]
mod tests;

use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, OnceLock};

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

pub(in crate::domain_computation::primary_graph) use super::application_attempt::{
    CompletedDecisionReuseProof, PreparedDecisionReuseContext,
};
pub use denial::{WorthQueryPriorOutputDenial, WorthQueryPriorOutputDenialKind};
pub(in crate::domain_computation::primary_graph) use input_cutoff::{
    cutoff_declines, prepare_stable_address, InputCutoffDecision, InputCutoffVerificationStop,
    PreparedInputCutoffBasis, PublishedStableLineage, StablePublicationStop,
};
pub(in crate::domain_computation::primary_graph) use input_reuse_key::PreparedInputReuseKey;
pub(in crate::domain_computation::primary_graph) use native_output_witness::{
    PreparedNativeOutputWitness, SealedNativeOutputWitness,
};
pub(in crate::domain_computation::primary_graph) use prepared_slot::prepare as prepare_output_lineage_slot;
pub(in crate::domain_computation::primary_graph) use prepared_slot::PreparedLineageRecoveryMetadata;
pub(in crate::domain_computation::primary_graph) use prepared_slot::PreparedOutputLineageSlot;
pub(in crate::domain_computation::primary_graph) use prepared_slot::{
    tree_insert_bytes, tree_work,
};
use recorded_output::{RecordedOutput, RecordedOutputMutable};
pub(in crate::domain_computation::primary_graph) use recorded_source_identity::RecordedSourceIdentity;
use resolution::latest_output_matching;
pub(in crate::domain_computation::primary_graph) use settlement_identity::RecordedSettlementIdentity;

use super::{
    provider::WorthQueryPrimaryGraphCommittedApplication, WorthQueryApplicationOutputCorrespondence,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct SemanticSource {
    runtime_authority: u64,
    schema: ApplicationSchemaBindingIdentity,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    output_binding: TypeId,
}

impl Ord for SemanticSource {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let key = |source: &Self| {
            (
                source.runtime_authority,
                source.schema.runtime_ordinal(),
                source.schema.generation(),
                *source.schema.package_identity(),
                *source.schema.schema_identity(),
                source.scope,
                source.output_binding,
            )
        };
        key(self).cmp(&key(other))
    }
}

impl PartialOrd for SemanticSource {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Default)]
pub(crate) struct WorthQueryApplicationOutputLineage {
    by_source: BTreeMap<
        SemanticSource,
        BTreeMap<
            worth_runtime_world::facade::ProductBranchIncarnation,
            BTreeMap<u64, RecordedGeneration>,
        >,
    >,
    partition_index: partition_index::OutputPartitionIndex,
    origins: BTreeMap<worth_runtime_world::facade::ProductBranchIncarnation, ProductCoordinate>,
    live_occurrences: BTreeSet<worth_runtime_world::facade::ProductBranchIncarnation>,
    cancelled_slots: Option<Box<prepared_slot::CancelledLineageSlot>>,
    output_families: HashMap<String, Vec<(TypeId, String)>>,
    retention: retained_capacity::LineageRetentionLedger,
}

type RecordedGeneration = Vec<Arc<OnceLock<RecordedOutput>>>;

pub(super) struct WorthQueryExactRecordedOutput {
    pub(super) correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(super) source_identity: RecordedSourceIdentity,
    pub(super) source_partition_identity: [u8; 32],
    pub(super) producer_dependency_identity: Option<[u8; 32]>,
    pub(super) idempotency_key_identity: [u8; 32],
    pub(super) runtime_authority: u64,
    pub(super) schema: ApplicationSchemaBindingIdentity,
    pub(super) scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(super) observed_source_facts:
        Arc<[super::application_attempt::WorthQueryApplicationObservedFact]>,
    pub(super) resources:
        Option<super::application_contribution::WorthQueryProducerDemandResources>,
}

#[derive(Clone)]
pub(super) struct WorthQueryProducerLineageHead {
    pub(super) occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    pub(super) dependency_identity: Option<[u8; 32]>,
    pub(super) idempotency_key_identity: [u8; 32],
    pub(super) settlement: Arc<RecordedSettlementIdentity>,
    /// A runtime performed origin permits execution-key replay. Checkpoint
    /// origins carry prior identity, but do not retain the native receipt.
    pub(super) may_replay_idempotency: bool,
    /// The head's record consumed upstream outputs. Only a row that posts its
    /// settlement holds the claims on them.
    pub(super) claims_upstream: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct ProductCoordinate {
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    generation: u64,
}

pub(super) struct WorthQueryPriorOutputBindingResolution {
    pub(super) correspondence: Option<Arc<WorthQueryApplicationOutputCorrespondence>>,
    pub(super) source_lookups: usize,
}

pub(super) struct WorthQueryCurrentOutputCandidate {
    pub(super) consumed_outputs: Arc<[super::invariant_projection::ConsumedOutputEvidence]>,
    pub(super) verification_requirement: Option<invalidation::FullVerificationReason>,
    pub(super) settlement_identity: Arc<RecordedSettlementIdentity>,
    pub(super) correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    /// The role the producer of this correspondence outputs to its family.
    pub(super) output_role: String,
    pub(super) observed_source_facts:
        Arc<[super::application_attempt::WorthQueryApplicationObservedFact]>,
    pub(super) native_output_witness: Option<Arc<OnceLock<SealedNativeOutputWitness>>>,
}

pub(super) struct WorthQueryCurrentOutputFamilyResolution {
    pub(super) family_installed: bool,
    pub(super) ambiguous_publication: bool,
    pub(super) candidates: Vec<WorthQueryCurrentOutputCandidate>,
    pub(super) selection_work: usize,
}

impl WorthQueryApplicationOutputLineage {
    pub(super) fn install_output_families(
        &mut self,
        families: BTreeMap<String, Vec<(TypeId, String)>>,
    ) {
        assert!(self.output_families.is_empty());
        self.output_families.extend(families);
    }

    pub(in crate::domain_computation::primary_graph) fn install_lineage_retention(
        &mut self,
        maximum_bytes: usize,
        history_positions: std::num::NonZeroUsize,
    ) {
        self.retention
            .install(maximum_bytes as u64, history_positions);
    }

    pub(crate) fn register_fork(
        &mut self,
        source: &worth_runtime_world::facade::ProductBranchObservation,
        destination: &worth_runtime_world::facade::ProductBranchObservation,
    ) {
        let source = ProductCoordinate {
            occurrence: source.lifecycle_incarnation(),
            generation: source.reference_generation().get(),
        };
        let destination = destination.lifecycle_incarnation();
        self.live_occurrences.insert(source.occurrence);
        self.live_occurrences.insert(destination);
        assert!(
            self.origins.insert(destination, source).is_none(),
            "one product occurrence may be registered as a fork once"
        );
    }
}
