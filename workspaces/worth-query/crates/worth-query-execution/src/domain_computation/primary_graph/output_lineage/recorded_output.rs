//! One retained correspondence; later restoration facts share its stable cell.

use std::sync::{Arc, Mutex, OnceLock};

use super::{
    invalidation::FullVerificationReason, retained_capacity::RetainedLineageCapacity,
    RecordedSettlementIdentity, RecordedSourceIdentity, WorthQueryApplicationOutputCorrespondence,
};
use crate::domain_computation::primary_graph::{
    application_attempt::{CompletedHandlerFactBoundary, WorthQueryApplicationObservedFact},
    application_contribution::WorthQueryProducerDemandResources,
    invariant_projection::ConsumedOutputEvidence,
};

pub(super) struct RecordedOutput {
    pub(super) _retained_capacity: Option<RetainedLineageCapacity>,
    /// Stable settlements pin the original performed record, never another
    /// alias. This preserves completion proof and its original lifetime cost.
    pub(super) performed_origin: Option<Arc<OnceLock<RecordedOutput>>>,
    pub(super) consumed_outputs: Arc<[ConsumedOutputEvidence]>,
    pub(super) completed_handler_facts: Option<CompletedHandlerFactBoundary>,
    pub(super) completed_decision_reuse: Option<super::CompletedDecisionReuseProof>,
    pub(super) prepared_input_reuse_key: Option<super::PreparedInputReuseKey>,
    /// Performed output initializes this slot at publication. Recovery may fill
    /// the same empty slot only after verifying its original checkpoint witness.
    pub(super) native_output_witness: OnceLock<Arc<OnceLock<super::SealedNativeOutputWitness>>>,
    pub(super) settlement_identity: Arc<RecordedSettlementIdentity>,
    pub(super) correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(super) source_identity: Option<RecordedSourceIdentity>,
    pub(super) source_partition_identity: Option<[u8; 32]>,
    pub(super) producer_dependency_identity: Option<[u8; 32]>,
    pub(super) idempotency_key_identity: [u8; 32],
    pub(super) mutable: Mutex<RecordedOutputMutable>,
}

pub(super) struct RecordedOutputMutable {
    pub(super) verification_requirement: Option<FullVerificationReason>,
    pub(super) observed_source_facts: Option<Arc<[WorthQueryApplicationObservedFact]>>,
    pub(super) resources: Option<WorthQueryProducerDemandResources>,
    /// What the partitioned computation of the attempt that published this
    /// record retained for the producer's next run.
    pub(super) computation: Option<super::retained_computation::RecordedComputation>,
}

impl RecordedOutput {
    pub(super) fn native_output_witness(&self) -> Option<&super::SealedNativeOutputWitness> {
        self.native_output_witness_cell()?.get()
    }

    pub(super) fn native_output_witness_cell(
        &self,
    ) -> Option<&Arc<OnceLock<super::SealedNativeOutputWitness>>> {
        self.native_output_witness.get()
    }
    pub(super) fn verification_requirement(&self) -> Option<FullVerificationReason> {
        self.mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .verification_requirement
    }

    pub(super) fn observed_source_facts(&self) -> Option<Arc<[WorthQueryApplicationObservedFact]>> {
        self.mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .observed_source_facts
            .clone()
    }

    /// The facts a checkpoint may carry for this output. A restored row is
    /// verified against its own source facts and claims nothing upstream, so
    /// an output that consumed others carries none: after a restore it starts
    /// Fresh and consumes its upstreams again, never a stale reuse.
    pub(super) fn checkpoint_source_facts(
        &self,
    ) -> Option<Arc<[WorthQueryApplicationObservedFact]>> {
        self.consumed_outputs
            .is_empty()
            .then(|| self.observed_source_facts())
            .flatten()
    }

    pub(super) fn resources(&self) -> Option<WorthQueryProducerDemandResources> {
        self.mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .resources
    }

    pub(super) fn restore(
        &self,
        facts: Arc<[WorthQueryApplicationObservedFact]>,
        resources: Option<WorthQueryProducerDemandResources>,
        verified_witness: Option<Arc<OnceLock<super::SealedNativeOutputWitness>>>,
    ) {
        if self.native_output_witness.get().is_some() {
            // A verified restoration keeps its first proof, the facts that
            // proof covers and the currentness established over them since.
            return;
        }
        let mut row = self
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        row.observed_source_facts = Some(facts);
        row.verification_requirement = Some(FullVerificationReason::CheckpointRestore);
        // A restored row's computation state is not the one its facts carry.
        row.computation = None;
        row.resources = resources;
        if let Some(witness) = verified_witness {
            // An initialized original witness is immutable. Repeated exact
            // restoration retains that first proof and its lifetime custody.
            let _ = self.native_output_witness.set(witness);
        }
    }

    pub(super) fn require_verification(&self, reason: FullVerificationReason) {
        self.mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .verification_requirement = Some(reason);
    }
}
