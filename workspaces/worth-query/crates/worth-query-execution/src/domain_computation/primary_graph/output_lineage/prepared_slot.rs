//! An invisible lineage address prepared before the World owner is contacted.

mod cancellation;
mod capacity;
mod preparation;
mod recovery;
use crate::domain_computation::primary_graph::application_contribution::SealedComputationRetention;
pub(super) use capacity::{arc_bytes, denial, tree_insert_bytes, tree_work};
pub(in crate::domain_computation::primary_graph) use preparation::prepare;
pub(in crate::domain_computation::primary_graph) use recovery::PreparedLineageRecoveryMetadata;

use std::sync::{Arc, Mutex, OnceLock};

use super::{
    ProductCoordinate, RecordedOutput, RecordedSettlementIdentity, SemanticSource,
    WorthQueryApplicationOutputLineage,
};
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication;

/// The identity stays invisible until the actual performed product head and
/// sealed output source match this reservation. Drop removes only its vacancy.
#[must_use = "a prepared output slot must be filled or released"]
pub(in crate::domain_computation::primary_graph) struct PreparedOutputLineageSlot {
    owner: Arc<Mutex<WorthQueryApplicationOutputLineage>>,
    pub(super) source: SemanticSource,
    pub(super) coordinate: ProductCoordinate,
    pub(super) partition: Option<[u8; 32]>,
    pub(super) identity: Arc<RecordedSettlementIdentity>,
    pub(super) record_cell: Arc<OnceLock<RecordedOutput>>,
    pub(super) partition_cell: Arc<OnceLock<usize>>,
    retained_capacity: Option<super::retained_capacity::RetainedLineageCapacity>,
    cancellation: Option<Box<CancelledLineageSlot>>,
    pub(super) completed_handler_facts: Option<
        crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary,
    >,
    pub(super) completed_decision_reuse: Option<super::CompletedDecisionReuseProof>,
    pub(super) prepared_input_reuse_key: Option<super::PreparedInputReuseKey>,
    pub(super) native_output_witness: Option<Arc<OnceLock<super::SealedNativeOutputWitness>>>,
    pub(super) actual_resources: Option<crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerDemandResources>,
    /// A sealed partitioned computation run and the record its prior state
    /// came from. World recovery carries both unchanged to its replacement slot.
    pub(super) computation: Option<SealedComputationRetention>,
    pub(super) computation_assigned: bool,
    pub(super) prior_computation: Option<super::PriorComputationRecord>,
    filled: bool,
}

pub(super) struct CancelledLineageSlot {
    identity: Arc<RecordedSettlementIdentity>,
    partition: Option<[u8; 32]>,
    retained_capacity: Option<super::retained_capacity::RetainedLineageCapacity>,
    pub(super) next: Option<Box<Self>>,
}

impl CancelledLineageSlot {
    pub(super) fn contains_generation(
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

    pub(super) fn contains(
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
pub(in crate::domain_computation::primary_graph) struct RecordedLineageSlot {
    pub(in crate::domain_computation::primary_graph) computation_source:
        super::ComputationSourceEvidence,
    pub(in crate::domain_computation::primary_graph) identity: Arc<RecordedSettlementIdentity>,
    pub(in crate::domain_computation::primary_graph) output_witness:
        Option<Arc<OnceLock<super::SealedNativeOutputWitness>>>,
    /// The settlement of the generation this record displaced as the latest
    /// output of its partition. Its row is this publication's to retire.
    pub(in crate::domain_computation::primary_graph) displaced:
        Option<Arc<RecordedSettlementIdentity>>,
}

impl PreparedOutputLineageSlot {
    pub(in crate::domain_computation::primary_graph) fn retain_actual_resources(
        &mut self,
        resources: Option<crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerDemandResources>,
    ) {
        assert!(self.actual_resources.is_none());
        self.actual_resources = resources;
    }

    pub(in crate::domain_computation::primary_graph) fn retain_completed_handler_facts(
        &mut self,
        boundary: crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary,
    ) {
        assert!(self.completed_handler_facts.is_none());
        self.completed_handler_facts = Some(boundary);
    }

    pub(in crate::domain_computation::primary_graph) fn retain_computation(
        &mut self,
        sealed: SealedComputationRetention,
        prior: Option<super::PriorComputationRecord>,
    ) {
        assert!(
            !self.computation_assigned,
            "a prepared slot retains completion once"
        );
        self.computation_assigned = true;
        self.computation = Some(sealed);
        self.prior_computation = prior;
    }

    pub(in crate::domain_computation::primary_graph) fn retain_completed_decision_reuse(
        &mut self,
        proof: super::CompletedDecisionReuseProof,
    ) {
        assert!(self.completed_decision_reuse.replace(proof).is_none());
    }

    pub(in crate::domain_computation::primary_graph) fn retain_prepared_input_reuse_key(
        &mut self,
        key: super::PreparedInputReuseKey,
    ) {
        assert!(self.prepared_input_reuse_key.is_none());
        self.prepared_input_reuse_key = Some(key);
    }

    pub(in crate::domain_computation::primary_graph) fn retain_native_output_witness(
        &mut self,
        witness: Arc<OnceLock<super::SealedNativeOutputWitness>>,
    ) {
        assert!(self.native_output_witness.is_none());
        self.native_output_witness = Some(witness);
    }

    pub(in crate::domain_computation::primary_graph) fn identity(
        &self,
    ) -> &Arc<RecordedSettlementIdentity> {
        &self.identity
    }

    pub(in crate::domain_computation::primary_graph) fn record(
        mut self,
        application: &WorthQueryPrimaryGraphCommittedApplication,
        consumed_outputs: Arc<[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence]>,
    ) -> RecordedLineageSlot {
        let output_witness = self.native_output_witness.as_ref().map(Arc::clone);
        let completed_handler_facts = self.completed_handler_facts.take();
        let completed_decision_reuse = self.completed_decision_reuse.take();
        let prepared_input_reuse_key = self.prepared_input_reuse_key.take();
        let retained_capacity = self
            .retained_capacity
            .take()
            .expect("prepared lineage retains host custody through publication");
        let (identity, displaced) = {
            let mut lineage = self
                .owner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let displaced = self.partition.and_then(|partition| {
                lineage.displaced_settlement(&self.source, self.coordinate, partition)
            });
            let sealed = self
                .computation
                .take()
                .expect("unfilled slot owns its total retention result");
            let computation = match sealed {
                SealedComputationRetention::Produced(sealed) => lineage.retain_computation(
                    sealed,
                    self.prior_computation.as_ref(),
                    displaced.as_ref(),
                ),
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
                retained_capacity,
                computation,
            );
            (identity, displaced)
        };
        self.filled = true;
        RecordedLineageSlot {
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
        cancellation.retained_capacity = self.retained_capacity.take();
        cancellation.next = lineage.cancelled_slots.take();
        lineage.cancelled_slots = Some(cancellation);
    }
}
