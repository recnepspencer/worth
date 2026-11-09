//! Product-local semantic output correspondence owned by Query publication.

pub(in crate::domain_computation) use invalidation::HeadCellRegistrationStop;
pub(crate) mod source_facts;
mod source_publication;
pub(in crate::domain_computation::primary_graph) use source_facts::ComparableSourceFacts;
pub(in crate::domain_computation::primary_graph) use source_facts::RetainedSourceFacts;
pub(in crate::domain_computation::primary_graph) use source_publication::CoveredSourcePublication;
pub(in crate::domain_computation::primary_graph) use source_publication::SourcePublication;
pub(crate) mod computation_source;
mod handler_preparation;
pub(in crate::domain_computation) use handler_preparation::ProducerHandlerPreparation;
mod implementation_build;
pub(in crate::domain_computation::primary_graph) use computation_source::ComputationSourceEvidence;
pub(in crate::domain_computation::primary_graph) use computation_source::CurrentComputation;
pub(crate) mod executed_adoption;
pub(in crate::domain_computation::primary_graph) use input_cutoff::executed_adoption::{
    publish as publish_executed_adoption, select as select_executed_adoption,
    ExecutedAdoptionSelection,
};
mod current_candidate;
pub(crate) mod current_output;
pub(in crate::domain_computation::primary_graph) use current_output::RetainedOutputCurrentnessRead;
mod denial;
pub(crate) mod family_selection;
pub(in crate::domain_computation::primary_graph) use family_selection::NativePriorCheckpointOutput;
pub(crate) mod input_cutoff;
pub(crate) mod input_reuse_key;
pub(crate) mod invalidation;
pub(crate) mod native_output_witness;
pub(crate) mod native_prior_checkpoint;

pub(crate) mod partition_index;
pub(crate) mod prepared_slot;

pub(in crate::domain_computation::primary_graph) use input_cutoff::RetainedInputCutoffCandidate;
pub(crate) mod coupled_completion;
mod qualification;
pub(crate) mod recorded_output;
pub(crate) mod recorded_source_identity;
pub(in crate::domain_computation::primary_graph) use coupled_completion::CoupledOutputCompletion;
pub(crate) mod required_publication;
pub(crate) mod required_settlement;
pub(in crate::domain_computation::primary_graph) use required_settlement::AcceptedCurrentCandidate;
pub(in crate::domain_computation::primary_graph) use required_settlement::BoundCurrentAcceptedOutput;
pub(in crate::domain_computation::primary_graph) use required_settlement::CurrentAcceptedResult;
pub(in crate::domain_computation::primary_graph) use required_settlement::CurrentAcceptedStop;
pub(in crate::domain_computation::primary_graph) use required_settlement::RequiredSettlementStop;
pub(crate) mod resolution;
mod resources;
pub(crate) mod restoration;
pub(in crate::domain_computation::primary_graph) use restoration::PriorOutputOrigin;
pub(crate) mod retained_computation;
pub(in crate::domain_computation::primary_graph) use retained_computation::PriorComputationRecord;
mod fork_origin;
mod historical_transport;
pub use historical_transport::{
    HistoricalAcceptedOrigin, WorthQueryHistoryPublicationCorrespondenceMismatch,
};
pub(in crate::domain_computation::primary_graph) use historical_transport::{
    HistoricalSelectionCoordinate, HistoricalTransportDenial, PreparedHistoricalTransport,
};
mod retention;
pub(crate) mod settlement_identity;

pub(in crate::domain_computation) use invalidation::SourceInvalidationOwner;

use super::output_family_identity::OutputFamilyIdentity;
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::OnceLock;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

pub(in crate::domain_computation::primary_graph) use super::application_attempt::CompletedDecisionReuseProof;
pub(in crate::domain_computation::primary_graph) use super::application_attempt::PreparedDecisionReuseContext;
pub use denial::WorthQueryPriorOutputDenial;
pub use denial::WorthQueryPriorOutputDenialKind;
pub(in crate::domain_computation::primary_graph) use input_cutoff::cutoff_declines;
pub(in crate::domain_computation::primary_graph) use input_cutoff::prepare_stable_address;
pub(in crate::domain_computation::primary_graph) use input_cutoff::InputCutoffVerificationStop;
pub(in crate::domain_computation::primary_graph) use input_cutoff::OwnedInputCutoffDecision;
pub(in crate::domain_computation::primary_graph) use input_cutoff::PreparedInputCutoffBasis;
pub(in crate::domain_computation::primary_graph) use input_cutoff::PublishedStableLineage;
pub(in crate::domain_computation::primary_graph) use input_cutoff::StablePublicationStop;
pub(in crate::domain_computation::primary_graph) use input_cutoff::VerifiedInputCutoff;
pub(in crate::domain_computation::primary_graph) use input_reuse_key::PreparedInputReuseKey;
pub(in crate::domain_computation::primary_graph) use native_output_witness::PreparedNativeOutputWitness;
pub(in crate::domain_computation::primary_graph) use native_output_witness::SealedNativeOutputWitness;
pub(in crate::domain_computation::primary_graph) use prepared_slot::prepare as prepare_output_lineage_slot;
pub(in crate::domain_computation::primary_graph) use prepared_slot::PreparedLineageRecoveryMetadata;
pub(in crate::domain_computation::primary_graph) use prepared_slot::PreparedOutputLineageSlot;
pub(in crate::domain_computation::primary_graph) use recorded_output::RecordedOutput;
use recorded_output::RecordedOutputMutable;
pub(in crate::domain_computation::primary_graph) use recorded_source_identity::RecordedSourceIdentity;
use resolution::latest_output_matching;
pub(in crate::domain_computation::primary_graph) use settlement_identity::RecordedSettlementIdentity;

use super::provider::WorthQueryPrimaryGraphCommittedApplication;
use super::WorthQueryApplicationOutputCorrespondence;

mod binding_inventory;
pub(crate) mod semantic_source;
pub(in crate::domain_computation::primary_graph) use semantic_source::SemanticSource;

#[derive(Default)]
pub struct WorthQueryApplicationOutputLineage {
    implementation_build: Option<worth_foundational::facade::ApplicationBuildIdentity>,
    by_source: BTreeMap<
        SemanticSource,
        BTreeMap<
            worth_runtime_world::facade::ProductBranchIncarnation,
            BTreeMap<u64, RecordedGeneration>,
        >,
    >,
    binding_identities: super::output_binding_identity::OutputBindingInventory,
    partition_index: partition_index::OutputPartitionIndex,
    origins: BTreeMap<worth_runtime_world::facade::ProductBranchIncarnation, ProductCoordinate>,
    live_occurrences: BTreeSet<worth_runtime_world::facade::ProductBranchIncarnation>,
    cancelled_slots: Option<Box<prepared_slot::CancelledLineageSlot>>,
    output_families: BTreeMap<OutputFamilyIdentity, Vec<(TypeId, String)>>,
    source_publication_coverage: source_publication::SourcePublicationCoverage,
}

type RecordedGeneration = Vec<Arc<OnceLock<RecordedOutput>>>;

pub struct WorthQueryExactRecordedOutput {
    pub(super) producing_implementation:
        Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation>,
    pub(super) correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(super) source_identity: RecordedSourceIdentity,
    pub(super) source_partition_identity: [u8; 32],
    pub(super) producer_dependency_identity: Option<[u8; 32]>,
    pub(super) idempotency_key_identity: [u8; 32],
    pub(super) runtime_authority: u64,
    pub(super) schema: ApplicationSchemaBindingIdentity,
    pub scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(super) observed_source_facts: RetainedSourceFacts,
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
    /// Runtime origins retain a replayable receipt; checkpoint origins retain prior identity.
    pub(super) may_replay_idempotency: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProductCoordinate {
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    generation: u64,
}

pub struct WorthQueryPriorOutputBindingResolution {
    pub(super) correspondence: Option<Arc<WorthQueryApplicationOutputCorrespondence>>,
}

pub struct WorthQueryCurrentOutputCandidate {
    pub(super) consumed_outputs: Arc<[super::invariant_projection::ConsumedOutputEvidence]>,
    pub(super) verification_requirement: Option<invalidation::FullVerificationReason>,
    pub(super) settlement_identity: Arc<RecordedSettlementIdentity>,
    pub(super) correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    /// The role the producer of this correspondence outputs to its family.
    pub(super) output_role: String,
    pub(super) observed_source_facts: RetainedSourceFacts,
    pub(super) native_output_witness: Option<Arc<OnceLock<SealedNativeOutputWitness>>>,
}

pub struct WorthQueryCurrentOutputFamilyResolution {
    pub(super) family_installed: bool,
    pub(super) ambiguous_publication: bool,
    pub(super) candidates: Vec<WorthQueryCurrentOutputCandidate>,
}

impl WorthQueryApplicationOutputLineage {
    pub(super) fn install_output_families(
        &mut self,
        families: BTreeMap<OutputFamilyIdentity, Vec<(TypeId, String)>>,
    ) {
        assert!(self.output_families.is_empty());
        self.output_families.extend(families);
        self.source_publication_coverage.install();
    }

    fn record_inner(
        &mut self,
        computation: retained_computation::RecordedComputation,
        application: &WorthQueryPrimaryGraphCommittedApplication,
        consumed_outputs: Arc<[super::invariant_projection::ConsumedOutputEvidence]>,
        prepared: Option<&PreparedOutputLineageSlot>,
        completed_handler_facts: Option<super::application_attempt::CompletedHandlerFactBoundary>,
        completed_decision_reuse: Option<CompletedDecisionReuseProof>,
        prepared_input_reuse_key: Option<PreparedInputReuseKey>,
        native_prior_checkpoint: Option<native_prior_checkpoint::NativePriorCheckpointLocator>,
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
        let fallback_source = if prepared.is_none() {
            Some(SemanticSource {
                runtime_authority: scope.runtime_authority(),
                schema: scope.binding_identity().clone(),
                scope: scope.scope(),
                output_binding: self.binding_identity(output_binding)?,
            })
        } else {
            None
        };
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
            assert_eq!(prepared.output_binding_type, output_binding);
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
        if prepared.is_none()
            && let Some(partition) = partition
        {
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
        // A commit whose rebase failed retains no fact, only the reason.
        let (observed_source_facts, verification_requirement) =
            match evidence.rebased_source_facts() {
                Ok(facts) => (Some(facts), None),
                Err(failed) => (None, Some(failed.reason.into())),
            };
        let computation_source = ComputationSourceEvidence::from_completed(application);
        let recorded = RecordedOutput {
            original_ready_key: prepared.and_then(|slot| slot.original_ready_key.clone()),
            coupled_completion: prepared.and_then(|slot| slot.coupled_completion.clone()),
            required_publication: prepared.and_then(|slot| slot.required_publication.clone()),
            native_prior_checkpoint,
            computation_source,
            performed_origin: None,
            historical_selection: None,
            consumed_outputs,
            completed_handler_facts,
            completed_decision_reuse,
            prepared_input_reuse_key,
            native_output_witness: prepared
                .and_then(|slot| slot.native_output_witness.as_ref().map(Arc::clone))
                .map(std::sync::OnceLock::from)
                .unwrap_or_default(),
            mutable: std::sync::Mutex::new(RecordedOutputMutable::new(
                verification_requirement,
                (observed_source_facts).map(|facts| computation_source.retain_facts(facts)),
                computation,
            )),
            settlement_identity: Arc::clone(&settlement_identity),
            correspondence: evidence.retain_output_correspondence(),
            source_identity: evidence.idempotency().source_identity().map(|identity| {
                RecordedSourceIdentity::Runtime(
                    super::application_query::WorthQueryRuntimeSourceIdentity::new(identity),
                )
            }),
            source_partition_identity: evidence.idempotency().source_partition_identity(),
            producer_dependency_identity: evidence.idempotency().producer_dependency_identity(),
            producing_implementation: evidence.idempotency().producing_implementation(),
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

mod court_doors_for_test;
