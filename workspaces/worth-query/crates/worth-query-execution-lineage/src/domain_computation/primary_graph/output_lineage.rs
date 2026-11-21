//! Product-local semantic output correspondence owned by Query publication.

pub use invalidation::HeadCellRegistrationStop;
pub mod checkpoint_prerequisites;
pub mod source_facts;
pub mod source_publication;
pub use source_facts::ComparableSourceFacts;
pub use source_facts::RetainedSourceFacts;
pub use source_publication::CoveredSourcePublication;
pub use source_publication::SourcePublication;
pub mod computation_source;
pub mod handler_preparation;
pub mod historical_reads;
pub use handler_preparation::ProducerHandlerPreparation;
pub mod implementation_build;
pub use computation_source::ComputationSourceEvidence;
pub use computation_source::CurrentComputation;
pub mod executed_adoption;
pub use input_cutoff::executed_adoption::{
    publish as publish_executed_adoption, select as select_executed_adoption,
    ExecutedAdoptionSelection,
};
pub mod current_candidate;
pub mod current_output;
pub mod original_publication;
pub use current_output::RetainedOutputCurrentnessRead;
pub mod denial;
pub mod family_selection;
pub use family_selection::NativePriorCheckpointOutput;
pub mod input_cutoff;
pub mod input_reuse_key;
pub mod invalidation;
pub mod native_output_witness;
pub mod native_prior_checkpoint;

pub mod partition_index;
pub mod prepared_slot;

pub use input_cutoff::RetainedInputCutoffCandidate;
pub mod coupled_completion;
pub mod qualification;
pub mod recorded_output;
pub mod recorded_source_identity;
pub use coupled_completion::CoupledOutputCompletion;
pub mod required_publication;
pub mod required_settlement;
pub use required_settlement::AcceptedCurrentCandidate;
pub use required_settlement::BoundCurrentAcceptedOutput;
pub use required_settlement::CurrentAcceptedResult;
pub use required_settlement::CurrentAcceptedStop;
pub use required_settlement::RequiredSettlementStop;
pub mod resolution;
pub mod resources;
pub mod restoration;
pub use restoration::PriorOutputOrigin;
pub mod retained_computation;
pub use retained_computation::PriorComputationRecord;
pub mod fork_origin;
pub mod historical_transport;
pub use historical_transport::{
    HistoricalAcceptedOrigin, WorthQueryHistoryPublicationCorrespondenceMismatch,
};
pub use historical_transport::{
    HistoricalSelectionCoordinate, HistoricalTransportDenial, PreparedHistoricalTransport,
};
pub mod retention;
pub mod settlement_identity;

pub use invalidation::SourceInvalidationOwner;

use super::output_family_identity::OutputFamilyIdentity;
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::OnceLock;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

pub use super::application_attempt::CompletedDecisionReuseProof;
pub use super::application_attempt::PreparedDecisionReuseContext;
pub use denial::WorthQueryPriorOutputDenial;
pub use denial::WorthQueryPriorOutputDenialKind;
pub use input_cutoff::cutoff_declines;
pub use input_cutoff::prepare_stable_address;
pub use input_cutoff::InputCutoffVerificationStop;
pub use input_cutoff::OwnedInputCutoffDecision;
pub use input_cutoff::PreparedInputCutoffBasis;
pub use input_cutoff::PublishedStableLineage;
pub use input_cutoff::StablePublicationStop;
pub use input_cutoff::VerifiedInputCutoff;
pub use input_reuse_key::PreparedInputReuseKey;
pub use native_output_witness::PreparedNativeOutputWitness;
pub use native_output_witness::SealedNativeOutputWitness;
pub use prepared_slot::prepare as prepare_output_lineage_slot;
pub use prepared_slot::PreparedLineageRecoveryMetadata;
pub use prepared_slot::PreparedOutputLineageSlot;
pub use recorded_output::RecordedOutput;
use recorded_output::RecordedOutputMutable;
pub use recorded_source_identity::RecordedSourceIdentity;
use resolution::latest_output_matching;
pub use settlement_identity::RecordedSettlementIdentity;

use super::provider::WorthQueryPrimaryGraphCommittedApplication;
use super::WorthQueryApplicationOutputCorrespondence;

pub mod binding_inventory;
pub mod semantic_source;
pub use semantic_source::SemanticSource;

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
    pub producing_implementation:
        Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation>,
    pub correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub source_identity: RecordedSourceIdentity,
    pub source_partition_identity: [u8; 32],
    pub producer_dependency_identity: Option<[u8; 32]>,
    pub idempotency_key_identity: [u8; 32],
    pub runtime_authority: u64,
    pub schema: ApplicationSchemaBindingIdentity,
    pub scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub observed_source_facts: RetainedSourceFacts,
}

#[derive(Clone)]
pub struct WorthQueryProducerLineageHead {
    pub occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    pub dependency_identity: Option<[u8; 32]>,
    pub idempotency_key_identity: [u8; 32],
    pub settlement: Arc<RecordedSettlementIdentity>,
    /// The head's record consumed upstream outputs. Only a row that posts its
    /// settlement holds the claims on them.
    pub claims_upstream: bool,
    /// Runtime origins retain a replayable receipt; checkpoint origins retain prior identity.
    pub may_replay_idempotency: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProductCoordinate {
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    generation: u64,
}

pub struct WorthQueryPriorOutputBindingResolution {
    pub correspondence: Option<Arc<WorthQueryApplicationOutputCorrespondence>>,
}

pub struct WorthQueryCurrentOutputCandidate {
    pub consumed_outputs: Arc<[super::invariant_projection::ConsumedOutputEvidence]>,
    pub verification_requirement: Option<invalidation::FullVerificationReason>,
    pub settlement_identity: Arc<RecordedSettlementIdentity>,
    pub correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    /// The role the producer of this correspondence outputs to its family.
    pub output_role: String,
    pub observed_source_facts: RetainedSourceFacts,
    pub native_output_witness: Option<Arc<OnceLock<SealedNativeOutputWitness>>>,
}

pub struct WorthQueryCurrentOutputFamilyResolution {
    pub family_installed: bool,
    pub ambiguous_publication: bool,
    pub candidates: Vec<WorthQueryCurrentOutputCandidate>,
}

impl WorthQueryApplicationOutputLineage {
    pub fn install_output_families(
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
