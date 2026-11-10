//! One retained correspondence; later restoration facts share its stable cell.

use std::sync::{Arc, Mutex, OnceLock};

use super::{
    invalidation::FullVerificationReason, RecordedSettlementIdentity, RecordedSourceIdentity,
    WorthQueryApplicationOutputCorrespondence,
};
use crate::domain_computation::primary_graph::{
    application_attempt::CompletedHandlerFactBoundary, invariant_projection::ConsumedOutputEvidence,
};

pub struct RecordedOutput {
    /// The actual managed registry key of the original standalone publication.
    pub(super) original_ready_key: Option<std::sync::Arc<crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey>>,
    pub(super) coupled_completion: Option<super::CoupledOutputCompletion>,
    pub(super) producing_implementation:
        Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation>,
    pub(super) required_publication:
        Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
    /// The performed computation read a fact that its own effect replaced.
    /// Postcondition facts cannot discharge this knowledge after index loss.
    pub(super) computation_source: super::ComputationSourceEvidence,
    pub(super) native_prior_checkpoint:
        Option<super::native_prior_checkpoint::NativePriorCheckpointLocator>,
    /// Stable settlements pin the original performed record, never another
    /// alias. This preserves completion proof and its original lifetime cost.
    pub(super) performed_origin: Option<Arc<OnceLock<RecordedOutput>>>,
    /// Set only on a row a graph selection commit transported: the selected
    /// and restored bases whose read evidence alone may admit its facts.
    pub(super) historical_selection: Option<Arc<super::HistoricalSelectionCoordinate>>,
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

pub struct RecordedOutputMutable {
    pub(super) verification_requirement: Option<FullVerificationReason>,
    observed_source_facts: Option<
        crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    >,
    stable_fact_spans: Option<super::input_cutoff::stable_publication::StableFactSpans>,
    /// What the partitioned computation of the attempt that published this
    /// record retained for the producer's next run.
    pub(super) computation: super::retained_computation::RecordedComputation,
}

impl RecordedOutputMutable {
    pub(super) fn new(
        verification_requirement: Option<FullVerificationReason>,
        facts: Option<super::RetainedSourceFacts>,
        computation: super::retained_computation::RecordedComputation,
    ) -> Self {
        match &computation {
            super::retained_computation::RecordedComputation::Retained(state) =>
                crate::domain_computation::primary_graph::application_contribution::observe_published(Some(state), None),
            super::retained_computation::RecordedComputation::Absent(reason) =>
                crate::domain_computation::primary_graph::application_contribution::observe_published(None, Some(reason.full_cause())),
        }
        Self {
            verification_requirement,
            observed_source_facts: facts.map(|facts| facts.postconditions().clone()),
            stable_fact_spans: None,
            computation,
        }
    }
    pub(super) fn replace_retained_facts(&mut self, facts: super::RetainedSourceFacts) {
        self.stable_fact_spans = None;
        self.observed_source_facts = Some(facts.postconditions().clone());
    }
    pub(super) fn with_stable_fact_spans(
        mut self,
        spans: super::input_cutoff::stable_publication::StableFactSpans,
    ) -> Self {
        self.set_stable_fact_spans(spans);
        self
    }
    pub(super) fn set_stable_fact_spans(
        &mut self,
        spans: super::input_cutoff::stable_publication::StableFactSpans,
    ) {
        assert!(self
            .observed_source_facts
            .as_ref()
            .is_some_and(|body| spans.matches(body)));
        self.stable_fact_spans = Some(spans);
    }
    /// Match immutable qualified custody while holding the row's mutable fence.
    pub(super) fn same_qualified_facts(
        &self,
        qualified: &super::ComparableSourceFacts,
    ) -> Option<super::ComparableSourceFacts> {
        self.observed_source_facts
            .as_ref()
            .filter(|facts| facts.same_backing(qualified.facts()))
            .map(|_| qualified.clone())
    }
}

impl RecordedOutput {
    /// Actual completed computation support is separate from canonical Native
    /// publication provenance. An adopting execution owns fresh support; an
    /// ordinary cutoff alias still borrows its original completed computation.
    pub(super) fn completed_computation_record(&self) -> Option<&Self> {
        if self.computation_source.is_executed_adoption() || self.performed_origin.is_none() {
            Some(self)
        } else {
            self.performed_origin
                .as_ref()?
                .get()
                .filter(|origin| origin.performed_origin.is_none())
        }
    }

    /// A record that keeps comparable facts is checked for currentness against
    /// its completed computation's sealed Native output witness. Without that
    /// witness the check cannot answer (public #126), and the record is not
    /// adoptable as current: the ordinary path runs the producer, as every
    /// other reader does for a record with no witness. The facts condition is
    /// the one the currentness read uses; a factless record is answered by its
    /// own commit and is never this.
    pub(super) fn retains_facts_without_witness(&self) -> bool {
        self.observed_source_facts()
            .and_then(|facts| facts.for_comparison())
            .is_some_and(|facts| !facts.is_empty())
            && self
                .completed_computation_record()
                .is_some_and(|completed| completed.native_output_witness().is_none())
    }

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

    pub(super) fn observed_source_facts(&self) -> Option<super::RetainedSourceFacts> {
        self.mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .observed_source_facts
            .clone()
            .map(|facts| super::RetainedSourceFacts::retain(self.computation_source, facts))
    }

    /// The facts a checkpoint may carry for this output. A restored row is
    /// verified against its own source facts and claims nothing upstream, so
    /// an output that consumed others carries none: after a restore it starts
    /// Fresh and consumes its upstreams again, never a stale reuse.
    pub(super) fn checkpoint_source_facts(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload>
    {
        self.consumed_outputs
            .is_empty()
            .then(|| self.observed_source_facts())
            .flatten()?
            .for_comparison()
            .map(|facts| facts.facts().clone())
    }

    pub(super) fn stable_fact_spans(
        &self,
        body: &crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    ) -> Option<super::input_cutoff::stable_publication::StableFactSpans> {
        let row = self
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        row.observed_source_facts
            .as_ref()
            .filter(|observed| observed.same_backing(body))?;
        row.stable_fact_spans
            .as_ref()
            .filter(|spans| spans.matches(body))
            .cloned()
    }

    pub(super) fn restore(
        &self,
        facts: crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
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
        row.stable_fact_spans = None;
        row.observed_source_facts = Some(facts);
        row.verification_requirement = Some(FullVerificationReason::CheckpointRestore);
        // A restored row's computation state is not the one its facts carry.
        row.computation = super::retained_computation::RecordedComputation::Absent(
            crate::domain_computation::primary_graph::application_contribution::PriorAbsence::Restored,
        );

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

#[cfg(test)]
mod span_tests;

mod adoption_basis_for_test;
mod court_doors_for_test;
