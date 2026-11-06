//! Original retained custody paired with the comparator's exact native pin.
//!
//! The read set holds this custody only through its own question,
//! `NativeCarriageReads`, which is answered here.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::read_set::computation_routing::NativeCarriageReads;
use crate::domain_computation::primary_graph::application_attempt::SealedComputationFacts;
use crate::domain_computation::primary_graph::application_contribution::RetainedComputation;
use worth_relational::facade::snapshots::SnapshotHandle;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::computation_routing::ComputationRead;

#[derive(Clone)]
pub struct NativeComputationReads {
    prior: Arc<RetainedComputation>,
    snapshot: SnapshotHandle,
    rebase_changes: Option<
        crate::domain_computation::primary_graph::output_lineage::invalidation::ComputationChanges,
    >,
}
impl NativeComputationReads {
    pub(super) fn call_binding(
        &self,
        sequence:&crate::domain_computation::primary_graph::invariant_projection::ComputationCallSequence,
    ) -> Option<crate::domain_computation::primary_graph::application_attempt::CheckedCallShape>
    {
        self.prior
            .checked_calls
            .as_ref()
            .filter(|binding| {
                self.prior.facts.has_call_shape(binding) && binding.same_shape(sequence.shape())
            })
            .cloned()
    }
    pub(super) fn new(prior: Arc<RetainedComputation>, snapshot: SnapshotHandle) -> Self {
        Self {
            prior,
            snapshot,
            rebase_changes: None,
        }
    }
    pub(super) fn with_rebase_changes(
        mut self,
        changes: Option<crate::domain_computation::primary_graph::output_lineage::invalidation::ComputationChanges>,
    ) -> Self {
        self.rebase_changes = changes;
        self
    }
}
impl NativeCarriageReads for NativeComputationReads {
    fn identity(&self) -> (*const (), &SnapshotHandle) {
        (Arc::as_ptr(&self.prior).cast::<()>(), &self.snapshot)
    }
    fn at_snapshot(&self, snapshot: &SnapshotHandle) -> bool {
        &self.snapshot == snapshot
    }
    fn facts(&self) -> &SealedComputationFacts {
        &self.prior.facts
    }
    fn rebase_changes(
        &self,
    ) -> Option<
        crate::domain_computation::primary_graph::output_lineage::invalidation::ComputationChanges,
    > {
        self.rebase_changes.clone()
    }
    fn closes_native_ranges(
        &self,
        completed: &CompletedComputationRetention,
        ranges: &Arc<
            crate::domain_computation::primary_graph::application_attempt::NativeCallCarriage,
        >,
    ) -> bool {
        let CompletedComputationRetention::Produced(run) = completed else {
            return false;
        };
        run.carried.as_ref().is_some_and(|carried| {
            Arc::ptr_eq(&carried.prior, &self.prior)
                && carried.validated_at.as_ref() == Some(&self.snapshot)
                && carried
                    .ranges
                    .as_ref()
                    .is_some_and(|closed| Arc::ptr_eq(closed, ranges))
                && ranges.same_native(self)
                && ranges.matches_target(run.call_sequence.shape())
        })
    }
    /// A descriptive layout cannot authorize carriage without the successful
    /// run that consumed this exact retained custody at the native pin.
    fn closes_native_reads(
        &self,
        completed: &CompletedComputationRetention,
        calls: &BTreeSet<ComputationRead>,
    ) -> bool {
        let CompletedComputationRetention::Produced(run) = completed else {
            return false;
        };
        run.carried.as_ref().is_some_and(|carried| {
            Arc::ptr_eq(&carried.prior, &self.prior)
                && carried.validated_at.as_ref() == Some(&self.snapshot)
                && carried.membership.then_some(ComputationRead::Membership).into_iter()
                    .chain(carried.items.iter().copied().map(ComputationRead::ItemKey))
                    .chain(carried.partitions.iter().map(ComputationRead::Partition))
                    .inspect(|_| crate::domain_computation::primary_graph::fact_verification_observation::completion_call_roster())
                    .eq(calls.iter().copied())
        })
    }
}
