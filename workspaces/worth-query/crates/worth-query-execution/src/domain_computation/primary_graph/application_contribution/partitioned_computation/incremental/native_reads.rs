//! Original retained custody paired with the comparator's exact native pin.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::SealedComputationFacts, output_lineage::CustodiedComputation,
};
use worth_relational::facade::snapshots::SnapshotHandle;

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct NativeComputationReads {
    prior: Arc<CustodiedComputation>,
    snapshot: SnapshotHandle,
}
impl NativeComputationReads {
    pub(super) fn new(prior: Arc<CustodiedComputation>, snapshot: SnapshotHandle) -> Self {
        Self { prior, snapshot }
    }
    pub(in crate::domain_computation::primary_graph) fn facts(&self) -> &SealedComputationFacts {
        &self.prior.facts
    }
    pub(in crate::domain_computation::primary_graph) fn at_snapshot(
        &self,
        snapshot: &SnapshotHandle,
    ) -> bool {
        &self.snapshot == snapshot
    }
}
impl CompletedComputationRetention {
    /// A descriptive layout cannot authorize carriage without the successful
    /// run that consumed this exact retained custody at the native pin.
    pub(in crate::domain_computation::primary_graph) fn closes_native_reads(
        &self,
        reads: &NativeComputationReads,
        calls: &BTreeSet<ComputationRead>,
    ) -> bool {
        let Self::Produced(run) = self else {
            return false;
        };
        run.carried.as_ref().is_some_and(|carried| {
            Arc::ptr_eq(&carried.prior, &reads.prior)
                && carried.validated_at.as_ref() == Some(&reads.snapshot)
                && carried.membership.then_some(ComputationRead::Membership).into_iter()
                    .chain(carried.items.iter().copied().map(ComputationRead::ItemKey))
                    .chain(carried.partitions.iter().map(ComputationRead::Partition))
                    .inspect(|_| crate::domain_computation::primary_graph::fact_verification_observation::completion_call_roster())
                    .eq(calls.iter().copied())
        })
    }
}
