//! The producer's prior and test-only handoff of completed runs.
#[cfg(test)]
use super::retained::SealedComputationRun;
use super::retained::WorthQueryPartitionedComputationFullCause;
use crate::domain_computation::primary_graph::application_contribution::InstalledProducerEdition;
use crate::domain_computation::primary_graph::output_lineage::{
    CustodiedComputation, PriorComputationRecord,
};
use std::sync::Arc;
/// What a producer hands the partitioned computation its handler runs: the
/// installed edition, the state the selected record retained, and that
/// record.
#[derive(Clone)]
pub(in crate::domain_computation) struct ComputationPrior {
    pub(super) edition: InstalledProducerEdition,
    pub(super) retained:
        Result<Arc<CustodiedComputation>, WorthQueryPartitionedComputationFullCause>,
    record: Option<PriorComputationRecord>,
    changes: Option<
        crate::domain_computation::primary_graph::output_lineage::invalidation::ComputationChanges,
    >,
}

impl ComputationPrior {
    pub(in crate::domain_computation::primary_graph) const fn new(
        edition: InstalledProducerEdition,
        retained: Result<Arc<CustodiedComputation>, WorthQueryPartitionedComputationFullCause>,
        record: Option<PriorComputationRecord>,
    ) -> Self {
        Self {
            edition,
            retained,
            record,
            changes: None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn observe_changes(
        &mut self,
        owner: &crate::domain_computation::primary_graph::SourceInvalidationOwner,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<(), worth_relational::facade::mvcc::CompanionPreflightStop> {
        if let (Some(record), Ok(state)) = (&self.record, &self.retained) {
            self.changes = record.changes(state, owner, runtime, snapshot, admission)?;
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn changed_ordinals(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Option<&im::OrdSet<usize>> {
        let changes = self.changes.as_ref()?;
        if !self.record.as_ref()?.matches_changes(changes) {
            return None;
        }
        changes.at_snapshot(runtime, snapshot)
    }
    /// The record the state was selected from.
    pub(in crate::domain_computation::primary_graph) fn into_record(
        self,
    ) -> Option<PriorComputationRecord> {
        self.record
    }
}

#[cfg(test)]
thread_local! {
    static PRIOR_IN_TEST: std::cell::RefCell<Option<ComputationPrior>> =
        const { std::cell::RefCell::new(None) };
    static SEALED_IN_TEST: std::cell::RefCell<Option<SealedComputationRun>> =
        const { std::cell::RefCell::new(None) };
}

/// An attempt no producer runs has no prior and keeps no run. A test hands
/// it both here.
#[cfg(test)]
impl ComputationPrior {
    pub(in crate::domain_computation) fn handed_in_test() -> Option<Self> {
        PRIOR_IN_TEST.with(|prior| prior.borrow().clone())
    }

    pub(super) fn hand_in_test(prior: Option<Self>) {
        PRIOR_IN_TEST.with(|handed| *handed.borrow_mut() = prior);
    }
}

#[cfg(test)]
impl SealedComputationRun {
    pub(in crate::domain_computation) fn keep_in_test(run: Option<Self>) {
        SEALED_IN_TEST.with(|kept| *kept.borrow_mut() = run);
    }

    pub(super) fn kept_in_test() -> Option<Self> {
        SEALED_IN_TEST.with(|kept| kept.borrow_mut().take())
    }
}
