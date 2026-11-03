//! Scripted test implementation of the production provider fault port.

use std::sync::atomic::AtomicU16;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::fault_port::WorthQueryPrimaryGraphFault;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::fault_port::WorthQueryPrimaryGraphFaultPort;

#[derive(Default)]
pub struct PrimaryGraphFaultController {
    pub scheduled: AtomicU16,
    pub staged_validation_cancellation: Mutex<Option<WorthQueryCancellationSource>>,
    pub native_preparation_cancellation:
        Mutex<Option<(WorthQueryCancellationSource, Option<std::time::Instant>)>>,
    pub native_preparation_interruptions: AtomicUsize,
    pub aftermath_interruption:
        Mutex<Option<(WorthQueryCancellationSource, Option<std::time::Instant>)>>,
    pub aftermath_interruptions: AtomicUsize,
    pub failed_post_commit_snapshot_consumptions: AtomicUsize,
    pub panicked_pending_publication_consumptions: AtomicUsize,
}

impl PrimaryGraphFaultController {
    pub fn interrupt_next_aftermath_seal(
        &self,
        source: WorthQueryCancellationSource,
        deadline: Option<std::time::Instant>,
    ) {
        *self.aftermath_interruption.lock().unwrap() = Some((source, deadline));
    }
    pub fn aftermath_interruption_count(&self) -> usize {
        self.aftermath_interruptions.load(Ordering::Acquire)
    }

    pub fn cancel_next_staged_validation(&self, source: WorthQueryCancellationSource) {
        *self.staged_validation_cancellation.lock().unwrap() = Some(source);
    }

    pub fn cancel_next_native_preparation(&self, source: WorthQueryCancellationSource) {
        *self.native_preparation_cancellation.lock().unwrap() = Some((source, None));
    }

    pub fn schedule(&self, fault: WorthQueryPrimaryGraphFault) {
        self.scheduled.fetch_or(mask(fault), Ordering::AcqRel);
    }

    pub fn lose_next_commit_response(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::LostCommitResponse);
    }

    pub fn reject_next_session_prepare(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::RejectedSessionPreparation);
    }

    pub fn reject_next_commit_before_transaction(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::RejectedCommitBeforeTransaction);
    }

    pub fn constrain_next_index_maintenance_budget(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::TightIndexMaintenanceBudget);
    }

    pub fn skip_next_invariant_owner_execution(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::SkippedInvariantOwnerExecution);
    }

    pub fn violate_next_relational_invariant(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::RelationalInvariantViolation);
    }

    pub fn fail_next_post_commit_snapshot(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::FailedPostCommitSnapshot);
    }

    pub fn panic_next_pending_application_publication(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::PanickedPendingApplicationPublication);
    }

    pub fn failed_post_commit_snapshot_consumption_count(&self) -> usize {
        self.failed_post_commit_snapshot_consumptions
            .load(Ordering::Acquire)
    }

    pub fn panicked_pending_publication_consumption_count(&self) -> usize {
        self.panicked_pending_publication_consumptions
            .load(Ordering::Acquire)
    }

    pub fn add_next_undeclared_application_touch(&self) {
        self.schedule(WorthQueryPrimaryGraphFault::UndeclaredApplicationTouch);
    }
}

impl WorthQueryPrimaryGraphFaultPort for PrimaryGraphFaultController {
    fn aftermath_ready_for_sealing(&self) {
        if let Some((source, deadline)) = self.aftermath_interruption.lock().unwrap().take() {
            self.aftermath_interruptions.fetch_add(1, Ordering::AcqRel);
            match deadline {
                None => source.cancel(),
                // Wait for this request's real deadline, at the sealing boundary.
                // No synthetic stop or timing race chooses the outcome.
                Some(deadline) => {
                    while let Some(remaining) =
                        deadline.checked_duration_since(std::time::Instant::now())
                    {
                        std::thread::sleep(remaining);
                    }
                }
            }
        }
    }

    fn candidate_ready_for_native_preparation(&self) {
        if let Some((source, deadline)) =
            self.native_preparation_cancellation.lock().unwrap().take()
        {
            self.native_preparation_interruptions
                .fetch_add(1, Ordering::AcqRel);
            match deadline {
                None => source.cancel(),
                Some(deadline) => {
                    while let Some(remaining) =
                        deadline.checked_duration_since(std::time::Instant::now())
                    {
                        std::thread::sleep(remaining);
                    }
                }
            }
        }
    }

    fn candidate_staged_for_validation(&self) {
        if let Some(source) = self.staged_validation_cancellation.lock().unwrap().take() {
            source.cancel();
        }
    }

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

pub const fn mask(fault: WorthQueryPrimaryGraphFault) -> u16 {
    match fault {
        WorthQueryPrimaryGraphFault::LostCommitResponse => 1 << 0,
        WorthQueryPrimaryGraphFault::RejectedSessionPreparation => 1 << 1,
        WorthQueryPrimaryGraphFault::RejectedCommitBeforeTransaction => 1 << 2,
        WorthQueryPrimaryGraphFault::SkippedInvariantOwnerExecution => 1 << 4,
        WorthQueryPrimaryGraphFault::RelationalInvariantViolation => 1 << 5,
        WorthQueryPrimaryGraphFault::FailedPostCommitSnapshot => 1 << 7,
        WorthQueryPrimaryGraphFault::DelayedOutputReadinessDelivery => 1 << 9,
        WorthQueryPrimaryGraphFault::UndeclaredApplicationTouch => 1 << 6,
        WorthQueryPrimaryGraphFault::PanickedPendingApplicationPublication => 1 << 8,
        WorthQueryPrimaryGraphFault::TightIndexMaintenanceBudget => 1 << 13,
        WorthQueryPrimaryGraphFault::FailedOutputReadinessEvaluation => 1 << 10,
        WorthQueryPrimaryGraphFault::ReadyReadSnapshotPressure => 1 << 11,
        WorthQueryPrimaryGraphFault::ReadinessSnapshotPressure => 1 << 12,
    }
}

#[path = "native_preparation_interruption/mod.rs"]
pub mod native_preparation_interruption;
