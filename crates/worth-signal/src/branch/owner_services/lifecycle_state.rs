use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use super::admission_table::SignalOwnerAdmissionTable;
use super::counters::SignalOwnerServiceCounters;
use super::operation_control::SignalOwnerOperationBoundary;
#[cfg(any(test, feature = "test-operation-control"))]
use super::operation_control::SignalOwnerOperationControl;
use super::SignalOwnerLifecycleObservation;

#[path = "lifecycle_state/cleanup_claim.rs"]
mod cleanup_claim;
#[path = "lifecycle_state/operation_admission.rs"]
mod operation_admission;
use cleanup_claim::SignalOwnerCleanupClaim;
pub(crate) use operation_admission::SignalOwnerOperationAdmission;
use operation_admission::SignalOwnerPendingAdmission;
pub(super) use operation_admission::{
    SignalOwnerBranchCellHold, SignalOwnerBranchCellHoldDenial, SignalOwnerMetadataHold,
    SignalOwnerMetadataHoldDenial,
};

static NEXT_SIGNAL_OWNER_LIFECYCLE_IDENTITY: AtomicU64 = AtomicU64::new(1);

const OWNER_PHASE_SHIFT: u32 = 62;
const OWNER_COUNT_MASK: u64 = (1_u64 << OWNER_PHASE_SHIFT) - 1;
const OWNER_OPEN: u64 = 0;
const OWNER_CLOSING: u64 = 1_u64 << OWNER_PHASE_SHIFT;
const OWNER_CLOSED: u64 = 2_u64 << OWNER_PHASE_SHIFT;

pub(crate) const MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalOwnerAdmissionDenial {
    ForeignOwner,
    OwnerUnavailable,
    OperationCapacityExhausted { maximum_in_flight_operations: usize },
    OwnerReentry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalOwnerAdmissionMismatch {
    ForeignOwner,
    ExpiredLifecycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalOwnerCloseDenial {
    ForeignOwner,
    ExecutingThreadHasAdmission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalOwnerLifecyclePoisonRecovery {
    PreservedLifecycleStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::branch) struct SignalOwnerLifecycleIdentity(u64);

pub(super) trait SignalOwnerCloseCoordinator {
    fn finish_owner_close(&self);
}

#[derive(Debug)]
pub(crate) struct SignalOwnerLifecycleState {
    owner_runtime_instance_id: u64,
    lifecycle_identity: SignalOwnerLifecycleIdentity,
    phase_and_count: AtomicU64,
    admission_table: Arc<SignalOwnerAdmissionTable>,
    transition_gate: Mutex<()>,
    drain: Condvar,
    cleanup_claimed: AtomicBool,
    #[cfg(test)]
    cleanup_waiters: std::sync::atomic::AtomicUsize,
    counters: Arc<SignalOwnerServiceCounters>,
    recovered_poison: AtomicBool,
    #[cfg(any(test, feature = "test-operation-control"))]
    operation_control: SignalOwnerOperationControl,
}

impl SignalOwnerLifecycleState {
    pub(crate) fn new(
        owner_runtime_instance_id: u64,
        counters: Arc<SignalOwnerServiceCounters>,
    ) -> Arc<Self> {
        Arc::new(Self {
            owner_runtime_instance_id,
            lifecycle_identity: next_lifecycle_identity(),
            phase_and_count: AtomicU64::new(OWNER_OPEN),
            admission_table: SignalOwnerAdmissionTable::new(),
            transition_gate: Mutex::new(()),
            drain: Condvar::new(),
            cleanup_claimed: AtomicBool::new(false),
            #[cfg(test)]
            cleanup_waiters: std::sync::atomic::AtomicUsize::new(0),
            counters,
            recovered_poison: AtomicBool::new(false),
            #[cfg(any(test, feature = "test-operation-control"))]
            operation_control: SignalOwnerOperationControl::new(),
        })
    }

    pub(crate) fn admit(
        self: &Arc<Self>,
        owner_runtime_instance_id: u64,
    ) -> Result<SignalOwnerOperationAdmission<'static>, SignalOwnerAdmissionDenial> {
        self.admit_with_close_coordinator_option(owner_runtime_instance_id, None)
    }

    pub(super) fn admit_with_close_coordinator<'owner>(
        self: &Arc<Self>,
        owner_runtime_instance_id: u64,
        close_coordinator: Arc<dyn SignalOwnerCloseCoordinator + 'owner>,
    ) -> Result<SignalOwnerOperationAdmission<'owner>, SignalOwnerAdmissionDenial> {
        self.admit_with_close_coordinator_option(owner_runtime_instance_id, Some(close_coordinator))
    }

    fn admit_with_close_coordinator_option<'owner>(
        self: &Arc<Self>,
        owner_runtime_instance_id: u64,
        close_coordinator: Option<Arc<dyn SignalOwnerCloseCoordinator + 'owner>>,
    ) -> Result<SignalOwnerOperationAdmission<'owner>, SignalOwnerAdmissionDenial> {
        if owner_runtime_instance_id != self.owner_runtime_instance_id {
            return Err(SignalOwnerAdmissionDenial::ForeignOwner);
        }
        let (reentry, scanned) = self.admission_table.executing_thread_has_owner_hold();
        self.counters.record_admission_records_scanned(scanned);
        if reentry {
            return Err(SignalOwnerAdmissionDenial::OwnerReentry);
        }
        let reservation = self.reserve_admission_count(close_coordinator)?;
        let record = self.admission_table.new_record();
        let (published, scanned) = self.admission_table.publish(record);
        self.counters.record_admission_records_scanned(scanned);
        let admission = SignalOwnerOperationAdmission::new(
            Arc::clone(self),
            owner_runtime_instance_id,
            self.lifecycle_identity,
            published,
            reservation.commit(),
        );
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::OwnerLifecycleAdmission);
        Ok(admission)
    }

    pub(crate) fn close(
        &self,
        owner_runtime_instance_id: u64,
    ) -> Result<(), SignalOwnerCloseDenial> {
        self.begin_close(owner_runtime_instance_id, true)?;
        loop {
            if let Some(claim) = self.claim_cleanup() {
                claim.complete();
            }
            if self.observation() == SignalOwnerLifecycleObservation::Closed {
                return Ok(());
            }
            self.wait_for_cleanup_turn();
        }
    }

    pub(super) fn begin_explicit_close(
        &self,
        owner_runtime_instance_id: u64,
    ) -> Result<(), SignalOwnerCloseDenial> {
        self.begin_close(owner_runtime_instance_id, true)
    }

    pub(super) fn request_close(
        &self,
        owner_runtime_instance_id: u64,
    ) -> Result<(), SignalOwnerCloseDenial> {
        self.begin_close(owner_runtime_instance_id, false)
    }

    pub(super) fn wait_until_closed(&self) {
        let mut gate = self.lock_transition_gate();
        while self.observation() != SignalOwnerLifecycleObservation::Closed {
            gate = match self.drain.wait(gate) {
                Ok(gate) => gate,
                Err(poisoned) => self.recover_poisoned_gate(poisoned),
            };
        }
    }

    pub(super) fn claim_cleanup(&self) -> Option<SignalOwnerCleanupClaim<'_>> {
        let _gate = self.lock_transition_gate();
        let state = self.phase_and_count.load(Ordering::Acquire);
        if state != OWNER_CLOSING
            || self
                .cleanup_claimed
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
        {
            return None;
        }
        Some(SignalOwnerCleanupClaim {
            lifecycle: self,
            completed: false,
        })
    }

    pub(super) fn wait_for_cleanup_turn(&self) {
        #[cfg(test)]
        self.cleanup_waiters.fetch_add(1, Ordering::AcqRel);
        let mut gate = self.lock_transition_gate();
        while self.observation() != SignalOwnerLifecycleObservation::Closed
            && (self.phase_and_count.load(Ordering::Acquire) != OWNER_CLOSING
                || self.cleanup_claimed.load(Ordering::Acquire))
        {
            gate = match self.drain.wait(gate) {
                Ok(gate) => gate,
                Err(poisoned) => self.recover_poisoned_gate(poisoned),
            };
        }
        #[cfg(test)]
        self.cleanup_waiters.fetch_sub(1, Ordering::AcqRel);
    }

    #[cfg(test)]
    pub(super) fn cleanup_waiter_count(&self) -> usize {
        self.cleanup_waiters.load(Ordering::Acquire)
    }

    pub(crate) fn observation(&self) -> SignalOwnerLifecycleObservation {
        match phase(self.phase_and_count.load(Ordering::Acquire)) {
            OWNER_OPEN => SignalOwnerLifecycleObservation::Open,
            OWNER_CLOSING => SignalOwnerLifecycleObservation::Closing,
            OWNER_CLOSED => SignalOwnerLifecycleObservation::Closed,
            _ => unreachable!("Signal owner lifecycle phase is packed by the owner"),
        }
    }

    pub(crate) fn owner_runtime_instance_id(&self) -> u64 {
        self.owner_runtime_instance_id
    }

    pub(in crate::branch) fn lifecycle_identity(&self) -> SignalOwnerLifecycleIdentity {
        self.lifecycle_identity
    }

    pub(crate) fn counters(&self) -> Arc<SignalOwnerServiceCounters> {
        Arc::clone(&self.counters)
    }

    pub(crate) fn reach_operation_boundary(&self, boundary: SignalOwnerOperationBoundary) {
        #[cfg(any(test, feature = "test-operation-control"))]
        self.operation_control.reach(boundary);
        #[cfg(not(any(test, feature = "test-operation-control")))]
        let _ = boundary;
    }

    #[cfg(any(test, feature = "test-operation-control"))]
    pub(super) fn operation_control(&self) -> SignalOwnerOperationControl {
        self.operation_control.clone()
    }

    pub(crate) fn poison_recovery(&self) -> Option<SignalOwnerLifecyclePoisonRecovery> {
        drop(self.lock_transition_gate());
        self.recovered_poison
            .load(Ordering::Acquire)
            .then_some(SignalOwnerLifecyclePoisonRecovery::PreservedLifecycleStatus)
    }

    fn reserve_admission_count<'owner>(
        self: &Arc<Self>,
        close_coordinator: Option<Arc<dyn SignalOwnerCloseCoordinator + 'owner>>,
    ) -> Result<SignalOwnerPendingAdmission<'owner>, SignalOwnerAdmissionDenial> {
        let mut observed = self.phase_and_count.load(Ordering::Acquire);
        loop {
            if phase(observed) != OWNER_OPEN {
                return Err(SignalOwnerAdmissionDenial::OwnerUnavailable);
            }
            let count = admission_count(observed);
            if count >= MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS {
                return Err(SignalOwnerAdmissionDenial::OperationCapacityExhausted {
                    maximum_in_flight_operations: MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS,
                });
            }
            match self.phase_and_count.compare_exchange_weak(
                observed,
                observed + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return Ok(SignalOwnerPendingAdmission {
                        lifecycle: Arc::clone(self),
                        close_coordinator,
                        committed: false,
                    })
                }
                Err(next) => observed = next,
            }
        }
    }

    fn release_operation(&self) -> bool {
        let _gate = self.lock_transition_gate();
        let mut observed = self.phase_and_count.load(Ordering::Acquire);
        loop {
            let count = admission_count(observed);
            debug_assert!(count > 0);
            let next = observed - 1;
            match self.phase_and_count.compare_exchange_weak(
                observed,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.drain.notify_all();
                    return next == OWNER_CLOSING;
                }
                Err(current) => observed = current,
            }
        }
    }

    fn begin_close(
        &self,
        owner_runtime_instance_id: u64,
        reject_executing_thread_admission: bool,
    ) -> Result<(), SignalOwnerCloseDenial> {
        if owner_runtime_instance_id != self.owner_runtime_instance_id {
            return Err(SignalOwnerCloseDenial::ForeignOwner);
        }
        if reject_executing_thread_admission {
            let (has_admission, scanned) = self.admission_table.executing_thread_has_admission();
            self.counters.record_admission_records_scanned(scanned);
            if has_admission {
                return Err(SignalOwnerCloseDenial::ExecutingThreadHasAdmission);
            }
        }
        let _gate = self.lock_transition_gate();
        let mut observed = self.phase_and_count.load(Ordering::Acquire);
        loop {
            if phase(observed) != OWNER_OPEN {
                return Ok(());
            }
            let next = OWNER_CLOSING | admission_count(observed) as u64;
            match self.phase_and_count.compare_exchange_weak(
                observed,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.drain.notify_all();
                    return Ok(());
                }
                Err(current) => observed = current,
            }
        }
    }

    fn lock_transition_gate(&self) -> MutexGuard<'_, ()> {
        match self.transition_gate.lock() {
            Ok(gate) => gate,
            Err(poisoned) => self.recover_poisoned_gate(poisoned),
        }
    }

    fn recover_poisoned_gate<'a>(
        &self,
        poisoned: std::sync::PoisonError<MutexGuard<'a, ()>>,
    ) -> MutexGuard<'a, ()> {
        self.recovered_poison.store(true, Ordering::Release);
        poisoned.into_inner()
    }

    #[cfg(test)]
    pub(super) fn poison_status_for_test(&self) {
        let _gate = self.lock_transition_gate();
        panic!("inject lifecycle-status poison");
    }
}

fn phase(state: u64) -> u64 {
    state & !OWNER_COUNT_MASK
}

fn admission_count(state: u64) -> usize {
    (state & OWNER_COUNT_MASK) as usize
}

fn next_lifecycle_identity() -> SignalOwnerLifecycleIdentity {
    let identity = NEXT_SIGNAL_OWNER_LIFECYCLE_IDENTITY
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .expect("Signal owner lifecycle identity exhausted");
    SignalOwnerLifecycleIdentity(identity)
}
