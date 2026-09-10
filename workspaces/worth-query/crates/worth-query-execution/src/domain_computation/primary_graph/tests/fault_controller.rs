//! Scripted test implementation of the production provider fault port.

use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};

use super::super::provider::fault_port::{
    WorthQueryPrimaryGraphFault, WorthQueryPrimaryGraphFaultPort,
};

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct PrimaryGraphFaultController {
    scheduled: AtomicU16,
    failed_post_commit_snapshot_consumptions: AtomicUsize,
    panicked_pending_publication_consumptions: AtomicUsize,
}

impl PrimaryGraphFaultController {
    pub(in crate::domain_computation::primary_graph) fn schedule(
        &self,
        fault: WorthQueryPrimaryGraphFault,
    ) {
        self.scheduled.fetch_or(mask(fault), Ordering::AcqRel);
    }

    pub(in crate::domain_computation::primary_graph) fn lose_next_commit_response(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::LostCommitResponse);
    }

    pub(in crate::domain_computation::primary_graph) fn reject_next_session_prepare(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::RejectedSessionPreparation);
    }

    pub(in crate::domain_computation::primary_graph) fn reject_next_commit_before_transaction(
        &self,
    ) {
        self.schedule(WorthQueryPrimaryGraphFault::RejectedCommitBeforeTransaction);
    }

    pub(in crate::domain_computation::primary_graph) fn fail_next_index_publication(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::FailedIndexPublication);
    }

    pub(in crate::domain_computation::primary_graph) fn skip_next_invariant_owner_execution(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::SkippedInvariantOwnerExecution);
    }

    pub(in crate::domain_computation::primary_graph) fn violate_next_relational_invariant(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::RelationalInvariantViolation);
    }

    pub(in crate::domain_computation::primary_graph) fn fail_next_post_commit_snapshot(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::FailedPostCommitSnapshot);
    }

    pub(in crate::domain_computation::primary_graph) fn panic_next_pending_application_publication(
        &self,
    ) {
        self.schedule(WorthQueryPrimaryGraphFault::PanickedPendingApplicationPublication);
    }

    pub(in crate::domain_computation::primary_graph) fn failed_post_commit_snapshot_consumption_count(
        &self,
    ) -> usize {
        self.failed_post_commit_snapshot_consumptions
            .load(Ordering::Acquire)
    }

    pub(in crate::domain_computation::primary_graph) fn panicked_pending_publication_consumption_count(
        &self,
    ) -> usize {
        self.panicked_pending_publication_consumptions
            .load(Ordering::Acquire)
    }

    pub(in crate::domain_computation::primary_graph) fn add_next_undeclared_application_touch(
        &self,
    ) {
        self.schedule(WorthQueryPrimaryGraphFault::UndeclaredApplicationTouch);
    }
}

impl WorthQueryPrimaryGraphFaultPort for PrimaryGraphFaultController {
    fn take(&self, fault: WorthQueryPrimaryGraphFault) -> bool {
        let mask = mask(fault);
        let mut scheduled = self.scheduled.load(Ordering::Acquire);
        loop {
            if scheduled & mask == 0 {
                return false;
            }
            match self.scheduled.compare_exchange_weak(
                scheduled,
                scheduled & !mask,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    if matches!(fault, WorthQueryPrimaryGraphFault::FailedPostCommitSnapshot) {
                        self.failed_post_commit_snapshot_consumptions
                            .fetch_add(1, Ordering::AcqRel);
                    }
                    if matches!(
                        fault,
                        WorthQueryPrimaryGraphFault::PanickedPendingApplicationPublication
                    ) {
                        self.panicked_pending_publication_consumptions
                            .fetch_add(1, Ordering::AcqRel);
                    }
                    return true;
                }
                Err(observed) => scheduled = observed,
            }
        }
    }
}

const fn mask(fault: WorthQueryPrimaryGraphFault) -> u16 {
    match fault {
        WorthQueryPrimaryGraphFault::LostCommitResponse => 1 << 0,
        WorthQueryPrimaryGraphFault::RejectedSessionPreparation => 1 << 1,
        WorthQueryPrimaryGraphFault::RejectedCommitBeforeTransaction => 1 << 2,
        WorthQueryPrimaryGraphFault::FailedIndexPublication => 1 << 3,
        WorthQueryPrimaryGraphFault::SkippedInvariantOwnerExecution => 1 << 4,
        WorthQueryPrimaryGraphFault::RelationalInvariantViolation => 1 << 5,
        WorthQueryPrimaryGraphFault::FailedPostCommitSnapshot => 1 << 7,
        WorthQueryPrimaryGraphFault::UndeclaredApplicationTouch => 1 << 6,
        WorthQueryPrimaryGraphFault::PanickedPendingApplicationPublication => 1 << 8,
    }
}
