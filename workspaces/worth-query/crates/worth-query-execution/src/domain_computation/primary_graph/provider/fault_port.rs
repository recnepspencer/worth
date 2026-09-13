//! Injected provider terminal-outcome boundary.

use std::sync::Arc;

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryPrimaryGraphFault {
    LostCommitResponse,
    RejectedSessionPreparation,
    RejectedCommitBeforeTransaction,
    FailedIndexPublication,
    SkippedInvariantOwnerExecution,
    RelationalInvariantViolation,
    FailedPostCommitSnapshot,
    DelayedOutputReadinessDelivery,
    #[cfg(test)]
    UndeclaredApplicationTouch,
    #[cfg(test)]
    PanickedPendingApplicationPublication,
}

pub(in crate::domain_computation::primary_graph) trait WorthQueryPrimaryGraphFaultPort:
    Send + Sync
{
    fn take(&self, fault: WorthQueryPrimaryGraphFault) -> bool;

    #[cfg(feature = "test-primary-graph-faults")]
    fn schedule_for_test(&self, _fault: WorthQueryPrimaryGraphFault) -> bool {
        false
    }
}

#[cfg(not(feature = "test-primary-graph-faults"))]
struct WorthQueryNoPrimaryGraphFaults;

#[cfg(not(feature = "test-primary-graph-faults"))]
impl WorthQueryPrimaryGraphFaultPort for WorthQueryNoPrimaryGraphFaults {
    fn take(&self, _fault: WorthQueryPrimaryGraphFault) -> bool {
        false
    }
}

pub(in crate::domain_computation::primary_graph) fn production_fault_port(
) -> Arc<dyn WorthQueryPrimaryGraphFaultPort> {
    #[cfg(feature = "test-primary-graph-faults")]
    {
        Arc::new(WorthQueryScriptedPrimaryGraphFaults::default())
    }
    #[cfg(not(feature = "test-primary-graph-faults"))]
    Arc::new(WorthQueryNoPrimaryGraphFaults)
}

#[cfg(feature = "test-primary-graph-faults")]
#[derive(Default)]
struct WorthQueryScriptedPrimaryGraphFaults {
    scheduled: std::sync::atomic::AtomicU16,
}

#[cfg(feature = "test-primary-graph-faults")]
impl WorthQueryPrimaryGraphFaultPort for WorthQueryScriptedPrimaryGraphFaults {
    fn take(&self, fault: WorthQueryPrimaryGraphFault) -> bool {
        let mask = fault_mask(fault);
        let mut scheduled = self.scheduled.load(std::sync::atomic::Ordering::Acquire);
        loop {
            if scheduled & mask == 0 {
                return false;
            }
            match self.scheduled.compare_exchange_weak(
                scheduled,
                scheduled & !mask,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(observed) => scheduled = observed,
            }
        }
    }

    fn schedule_for_test(&self, fault: WorthQueryPrimaryGraphFault) -> bool {
        self.scheduled
            .fetch_or(fault_mask(fault), std::sync::atomic::Ordering::AcqRel);
        true
    }
}

#[cfg(feature = "test-primary-graph-faults")]
const fn fault_mask(fault: WorthQueryPrimaryGraphFault) -> u16 {
    match fault {
        WorthQueryPrimaryGraphFault::LostCommitResponse => 1 << 0,
        WorthQueryPrimaryGraphFault::RejectedSessionPreparation => 1 << 1,
        WorthQueryPrimaryGraphFault::RejectedCommitBeforeTransaction => 1 << 2,
        WorthQueryPrimaryGraphFault::FailedIndexPublication => 1 << 3,
        WorthQueryPrimaryGraphFault::SkippedInvariantOwnerExecution => 1 << 4,
        WorthQueryPrimaryGraphFault::RelationalInvariantViolation => 1 << 5,
        WorthQueryPrimaryGraphFault::FailedPostCommitSnapshot => 1 << 7,
        WorthQueryPrimaryGraphFault::DelayedOutputReadinessDelivery => 1 << 9,
        #[cfg(test)]
        WorthQueryPrimaryGraphFault::UndeclaredApplicationTouch => 1 << 6,
        #[cfg(test)]
        WorthQueryPrimaryGraphFault::PanickedPendingApplicationPublication => 1 << 8,
    }
}

impl super::WorthQueryPrimaryGraphProvider {
    pub(super) fn take_lost_commit_response(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::LostCommitResponse)
    }

    pub(super) fn take_rejected_session_prepare(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::RejectedSessionPreparation)
    }

    pub(super) fn take_rejected_commit_before_transaction(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::RejectedCommitBeforeTransaction)
    }

    pub(super) fn take_failed_index_publication(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::FailedIndexPublication)
    }

    pub(super) fn take_failed_post_commit_snapshot(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::FailedPostCommitSnapshot)
    }

    #[cfg(feature = "test-primary-graph-faults")]
    pub(in crate::domain_computation::primary_graph) fn fail_next_index_publication_for_test(
        &self,
    ) {
        assert!(self
            .fault_port
            .schedule_for_test(WorthQueryPrimaryGraphFault::FailedIndexPublication));
    }

    pub(super) fn take_skipped_invariant_owner_execution(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::SkippedInvariantOwnerExecution)
    }

    pub(super) fn take_relational_invariant_violation(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::RelationalInvariantViolation)
    }

    #[cfg(test)]
    pub(super) fn take_undeclared_application_touch(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::UndeclaredApplicationTouch)
    }

    #[cfg(test)]
    pub(super) fn take_panicked_pending_application_publication(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::PanickedPendingApplicationPublication)
    }

    pub(super) fn take_fault(&self, fault: WorthQueryPrimaryGraphFault) -> bool {
        self.fault_port.take(fault)
    }
}
