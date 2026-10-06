//! Product-local semantic output correspondence owned by Query publication.

mod current_output;
pub(in crate::domain_computation::primary_graph) use current_output::RetainedOutputCurrentnessRead;
mod denial;
mod family_selection;
mod input_cutoff;
mod input_reuse_key;
pub(in crate::domain_computation::primary_graph) mod invalidation;
mod native_output_witness;
mod partition_index;
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
    pub(super) candidates: Vec<WorthQueryCurrentOutputCandidate>,
    pub(super) source_lookups: usize,
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

    fn record_prepared(
        &mut self,
        application: &WorthQueryPrimaryGraphCommittedApplication,
        consumed_outputs: Arc<[super::invariant_projection::ConsumedOutputEvidence]>,
        prepared: &PreparedOutputLineageSlot,
        completed_handler_facts: Option<super::application_attempt::CompletedHandlerFactBoundary>,
        completed_decision_reuse: Option<CompletedDecisionReuseProof>,
        prepared_input_reuse_key: Option<PreparedInputReuseKey>,
        retained_capacity: retained_capacity::RetainedLineageCapacity,
    ) -> Arc<RecordedSettlementIdentity> {
        self.record_inner(
            application,
            consumed_outputs,
            Some(prepared),
            completed_handler_facts,
            completed_decision_reuse,
            prepared_input_reuse_key,
            Some(retained_capacity),
        )
        .expect("a prepared output slot has a sealed output binding")
    }

    fn record_inner(
        &mut self,
        application: &WorthQueryPrimaryGraphCommittedApplication,
        consumed_outputs: Arc<[super::invariant_projection::ConsumedOutputEvidence]>,
        prepared: Option<&PreparedOutputLineageSlot>,
        completed_handler_facts: Option<super::application_attempt::CompletedHandlerFactBoundary>,
        completed_decision_reuse: Option<CompletedDecisionReuseProof>,
        prepared_input_reuse_key: Option<PreparedInputReuseKey>,
        retained_capacity: Option<retained_capacity::RetainedLineageCapacity>,
    ) -> Option<Arc<RecordedSettlementIdentity>> {
        let evidence = application.commit_evidence();
        let correspondence = evidence.output_correspondence();
        let scope = evidence.operation_scope();
        let publication = application.committed_product_publication();
        let coordinate = ProductCoordinate {
            occurrence: publication.product_incarnation(),
            generation: publication.product_generation().get(),
        };
        let output_binding = correspondence.binding_type()?;
        // The performed path borrows the pre-effect source. Constructing a
        // fresh source here could allocate after World has moved.
        let fallback_source = prepared.is_none().then(|| SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding,
        });
        let source = prepared.map_or_else(
            || {
                fallback_source
                    .as_ref()
                    .expect("test-only record owns its source")
            },
            |prepared| &prepared.source,
        );
        let partition = evidence.idempotency().source_partition_identity();
        if let Some(prepared) = prepared {
            assert_eq!(source.runtime_authority, scope.runtime_authority());
            assert_eq!(&source.schema, scope.binding_identity());
            assert_eq!(source.scope, scope.scope());
            assert_eq!(source.output_binding, output_binding);
            assert_eq!(
                prepared.coordinate, coordinate,
                "World performed the prepared product address"
            );
            assert_eq!(
                prepared.partition, partition,
                "prepared partition matches sealed commit evidence"
            );
        } else {
            self.live_occurrences.insert(coordinate.occurrence);
        }
        if prepared.is_none() {
            if let Some(partition) = partition {
                assert!(
                    self.partition_index
                        .at_generation(
                            source,
                            coordinate.occurrence,
                            coordinate.generation,
                            partition
                        )
                        .is_none(),
                    "one product generation may publish one output binding once"
                );
            }
        }
        let generation = prepared.is_none().then(|| {
            self.by_source
                .entry(source.clone())
                .or_default()
                .entry(coordinate.occurrence)
                .or_default()
                .entry(coordinate.generation)
                .or_default()
        });
        if partition.is_none() {
            assert!(
                generation.as_ref().is_none_or(|generation| {
                    generation.iter().all(|cell| {
                        cell.get()
                            .is_some_and(|recorded| recorded.source_partition_identity.is_some())
                    })
                }),
                "one product generation may publish one output binding once"
            );
        }
        let slot = prepared.map_or_else(
            || generation.as_ref().unwrap().len(),
            |slot| slot.identity.slot(),
        );
        let settlement_identity = if let Some(prepared) = prepared {
            assert!(prepared.record_cell.get().is_none());
            Arc::clone(&prepared.identity)
        } else {
            RecordedSettlementIdentity::retain(source, coordinate, slot)
        };
        let recorded = RecordedOutput {
            performed_origin: None,
            _retained_capacity: retained_capacity,
            consumed_outputs,
            completed_handler_facts,
            completed_decision_reuse,
            prepared_input_reuse_key,
            native_output_witness: prepared
                .and_then(|slot| slot.native_output_witness.as_ref().map(Arc::clone))
                .map(std::sync::OnceLock::from)
                .unwrap_or_default(),
            mutable: std::sync::Mutex::new(RecordedOutputMutable {
                verification_requirement: evidence.source_fact_verification_requirement().map(
                    |reason| match reason {
                        super::provider::RebaseVerificationReason::NativeRevisionUnavailable => {
                            invalidation::FullVerificationReason::NativeRevisionUnavailable
                        }
                        super::provider::RebaseVerificationReason::NativeFactRevisionUnavailable(ordinal) => {
                            invalidation::FullVerificationReason::NativeFactRevisionUnavailable(ordinal)
                        }
                        super::provider::RebaseVerificationReason::UnsupportedDecisionFact => {
                            invalidation::FullVerificationReason::UnsupportedFact
                        }
                        super::provider::RebaseVerificationReason::AdmissionDenied(stop) => {
                            invalidation::FullVerificationReason::MarkingAdmissionDenied(stop)
                        }
                    },
                ),
                observed_source_facts: evidence
                    .retain_observed_source_facts()
                    .or_else(|| evidence.retain_verification_source_facts()),
                resources: prepared.and_then(|slot| slot.actual_resources),
            }),
            settlement_identity: Arc::clone(&settlement_identity),
            correspondence: evidence.retain_output_correspondence(),
            source_identity: evidence.idempotency().source_identity().map(|identity| {
                RecordedSourceIdentity::Runtime(
                    super::application_query::WorthQueryRuntimeSourceIdentity::new(identity),
                )
            }),
            source_partition_identity: evidence.idempotency().source_partition_identity(),
            producer_dependency_identity: evidence.idempotency().producer_dependency_identity(),
            idempotency_key_identity: *evidence.idempotency().key_identity(),
        };
        if let Some(prepared) = prepared {
            assert!(
                prepared.record_cell.set(recorded).is_ok(),
                "prepared output fills once"
            );
            assert!(
                prepared.partition_cell.set(slot).is_ok(),
                "prepared partition fills once"
            );
        } else {
            let cell = Arc::new(OnceLock::new());
            assert!(cell.set(recorded).is_ok());
            generation.unwrap().push(cell);
            self.partition_index.insert(
                source.clone(),
                coordinate.occurrence,
                coordinate.generation,
                partition,
                slot,
            );
        }
        Some(settlement_identity)
    }
}
