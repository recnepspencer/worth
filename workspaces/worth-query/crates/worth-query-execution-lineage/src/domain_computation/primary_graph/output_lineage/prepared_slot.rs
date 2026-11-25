//! An invisible lineage address prepared before the World owner is contacted.

pub mod cancellation;

pub mod capacity;
pub mod checkpoint_locator;
pub mod computation;
use crate::equality_type_id::EqualityTypeId as TypeId;
use computation::PreparedComputationCustody;
pub mod preparation;
pub mod recovery;
use crate::domain_computation::primary_graph::application_contribution::SealedComputationRetention;
pub use capacity::denial;

pub use preparation::prepare;
pub use recovery::PreparedLineageRecoveryMetadata;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;

use super::ProductCoordinate;
use super::RecordedOutput;
use super::RecordedSettlementIdentity;
use super::SemanticSource;
use super::WorthQueryApplicationOutputLineage;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication;

/// The identity stays invisible until the actual performed product head and
/// sealed output source match this reservation. Drop removes only its vacancy.
#[must_use = "a prepared output slot must be filled or released"]
pub struct PreparedOutputLineageSlot {
    /// The actual managed registry key of the original standalone publication.
    pub original_ready_key: Option<std::sync::Arc<crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey>>,
    pub coupled_completion: Option<super::CoupledOutputCompletion>,
    pub required_publication:
        Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
    owner: super::SharedOutputLineage,
    pub source: SemanticSource,
    pub output_binding_type: TypeId,
    pub coordinate: ProductCoordinate,
    pub partition: Option<[u8; 32]>,
    pub identity: Arc<RecordedSettlementIdentity>,
    pub record_cell: Arc<OnceLock<RecordedOutput>>,
    pub partition_cell: Arc<OnceLock<usize>>,
    cancellation: Option<Box<CancelledLineageSlot>>,
    pub completed_handler_facts: Option<
        crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary,
    >,
    pub completed_decision_reuse: Option<super::CompletedDecisionReuseProof>,
    pub prepared_input_reuse_key: Option<super::PreparedInputReuseKey>,
    pub native_output_witness: Option<Arc<OnceLock<super::SealedNativeOutputWitness>>>,
    /// A sealed partitioned computation run and the record its prior state
    /// came from. World recovery carries both unchanged to its replacement slot.
    pub computation: PreparedComputationCustody,
    /// A pre-publication work allowance; a larger current fork set is not scanned.
    pub prior_computation: Option<super::PriorComputationRecord>,
    pub native_prior_checkpoint:
        Option<super::native_prior_checkpoint::NativePriorCheckpointLocator>,
    filled: bool,
}

pub struct CancelledLineageSlot {
    identity: Arc<RecordedSettlementIdentity>,
    partition: Option<[u8; 32]>,
    pub next: Option<Box<Self>>,
}

impl CancelledLineageSlot {
    pub fn contains_generation(
        head: &Option<Box<Self>>,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
    ) -> bool {
        let mut next = head.as_deref();
        while let Some(cue) = next {
            if cue.identity.source() == source && cue.identity.coordinate() == coordinate {
                return true;
            }
            next = cue.next.as_deref();
        }
        false
    }

    pub fn contains(
        head: &Option<Box<Self>>,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: Option<[u8; 32]>,
    ) -> bool {
        let mut next = head.as_deref();
        while let Some(cue) = next {
            if cue.identity.source() == source
                && cue.identity.coordinate() == coordinate
                && cue.partition == partition
            {
                return true;
            }
            next = cue.next.as_deref();
        }
        false
    }
}

/// What filling a prepared slot hands its publication.
pub struct RecordedLineageSlot {
    pub computation_registration: Option<super::invalidation::ComputationRegistration>,
    pub computation_source: super::ComputationSourceEvidence,
    pub identity: Arc<RecordedSettlementIdentity>,
    pub output_witness: Option<Arc<OnceLock<super::SealedNativeOutputWitness>>>,
    /// The settlement of the generation this record displaced as the latest
    /// output of its partition. Its row is this publication's to retire.
    pub displaced: Option<Arc<RecordedSettlementIdentity>>,
}

impl PreparedOutputLineageSlot {
    pub fn retain_original_ready_key(
        &mut self,
        key: std::sync::Arc<crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey>,
    ) {
        assert!(self.original_ready_key.is_none());
        self.original_ready_key = Some(key);
    }

    pub fn retain_completed_handler_facts(
        &mut self,
        boundary: crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary,
    ) {
        assert!(self.completed_handler_facts.is_none());
        self.completed_handler_facts = Some(boundary);
    }

    pub fn retain_computation(
        &mut self,
        sealed: SealedComputationRetention,
        prior: Option<super::PriorComputationRecord>,
    ) {
        self.computation.assign(sealed);
        self.prior_computation = prior;
    }

    pub fn retain_completed_decision_reuse(&mut self, proof: super::CompletedDecisionReuseProof) {
        assert!(self.completed_decision_reuse.replace(proof).is_none());
    }

    pub fn retain_prepared_input_reuse_key(&mut self, key: super::PreparedInputReuseKey) {
        assert!(self.prepared_input_reuse_key.is_none());
        self.prepared_input_reuse_key = Some(key);
    }

    pub fn retain_native_output_witness(
        &mut self,
        witness: Arc<OnceLock<super::SealedNativeOutputWitness>>,
    ) {
        assert!(self.native_output_witness.is_none());
        self.native_output_witness = Some(witness);
    }

    pub fn retain_coupled_completion(&mut self, completion: super::CoupledOutputCompletion) {
        assert!(self.coupled_completion.replace(completion).is_none());
    }
    pub fn seal_published_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        after: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial> {
        if let Some(completion) = &self.coupled_completion {
            completion
                .source()
                .seal_published(execution, runtime, after)?;
        }
        Ok(())
    }
    pub fn identity(&self) -> &Arc<RecordedSettlementIdentity> {
        &self.identity
    }

    pub fn record(
        mut self,
        application: &WorthQueryPrimaryGraphCommittedApplication,
        consumed_outputs: Arc<[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence]>,
    ) -> RecordedLineageSlot {
        if let Some(completion) = &self.coupled_completion {
            completion.bind_lineage(&self);
        }
        let output_witness = self.native_output_witness.as_ref().map(Arc::clone);
        let completed_handler_facts = self.completed_handler_facts.take();
        let completed_decision_reuse = self.completed_decision_reuse.take();
        let prepared_input_reuse_key = self.prepared_input_reuse_key.take();
        let native_prior_checkpoint = self.native_prior_checkpoint.take();
        let (identity, displaced) = {
            let mut lineage = self
                .owner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let displaced = self.partition.and_then(|partition| {
                lineage.displaced_settlement(&self.source, self.coordinate, partition)
            });
            let sealed = self.computation.take();
            let computation = match sealed {
                SealedComputationRetention::Produced(sealed) => lineage.retain_computation(sealed),
                SealedComputationRetention::Absent(absence) => {
                    super::retained_computation::RecordedComputation::Absent(absence)
                }
            };
            let identity = lineage.record_prepared(
                application,
                consumed_outputs,
                &self,
                completed_handler_facts,
                completed_decision_reuse,
                prepared_input_reuse_key,
                native_prior_checkpoint,
                computation,
            );
            (identity, displaced)
        };
        self.filled = true;
        let computation_registration = self.record_cell.get().and_then(|record| {
            let row = record
                .mutable
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            matches!(
                row.computation,
                super::retained_computation::RecordedComputation::Retained(_)
            )
            .then(super::invalidation::ComputationRegistration::retained)
        });
        RecordedLineageSlot {
            computation_registration,
            computation_source: self
                .record_cell
                .get()
                .expect("performed slot is filled")
                .computation_source,
            identity,
            output_witness,
            displaced,
        }
    }
}

impl Drop for PreparedOutputLineageSlot {
    fn drop(&mut self) {
        if self.filled {
            return;
        }
        let mut lineage = self
            .owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut cancellation = self.cancellation.take().expect("prepared cancellation cue");
        cancellation.next = lineage.cancelled_slots.take();
        lineage.cancelled_slots = Some(cancellation);
    }
}

impl WorthQueryApplicationOutputLineage {
    pub fn record_prepared(
        &mut self,
        application: &WorthQueryPrimaryGraphCommittedApplication,
        consumed_outputs: Arc<[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence]>,
        prepared: &PreparedOutputLineageSlot,
        completed_handler_facts: Option<crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary>,
        completed_decision_reuse: Option<super::CompletedDecisionReuseProof>,
        prepared_input_reuse_key: Option<super::PreparedInputReuseKey>,
        native_prior_checkpoint: Option<
            super::native_prior_checkpoint::NativePriorCheckpointLocator,
        >,
        computation: super::retained_computation::RecordedComputation,
    ) -> Arc<RecordedSettlementIdentity> {
        self.record_inner(
            computation,
            application,
            consumed_outputs,
            Some(prepared),
            completed_handler_facts,
            completed_decision_reuse,
            prepared_input_reuse_key,
            native_prior_checkpoint,
        )
        .expect("a prepared output slot has a sealed output binding")
    }
}

mod court_doors_for_test;
