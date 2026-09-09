use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

use crate::branch::SignalBranchRetirementReceipt;
use crate::state::SignalBranchId;

use super::branch_registry::SignalBranchCellConstruction;
use super::cancellation::SignalOwnerMovementPermit;
use super::cell_incarnation::SignalBranchCellIncarnation;
use super::counters::SignalOwnerServiceCounters;
use super::lifecycle_state::{
    SignalOwnerAdmissionMismatch, SignalOwnerBranchCellHoldDenial, SignalOwnerLifecycleIdentity,
    SignalOwnerOperationAdmission,
};

pub(crate) mod advance;
pub(crate) mod basis;
mod committed_patch_delivery;
mod conditional_execution;
mod conditional_installation;
mod conditional_issuance;
mod conditional_owned_async;
mod conditional_retirement;
mod conditional_topology;
pub(crate) mod fork;
#[path = "branch_execution_cell/fork_custody.rs"]
mod fork_custody;
mod inspection;
pub(crate) mod restoration;
pub(crate) mod retirement;
pub(crate) mod retirement_planning;
pub(crate) mod snapshot;

pub(in crate::branch::owner_services) use fork_custody::SignalBranchForkSourceCustody;

const CELL_LIVE: u8 = 0;
const CELL_RETIRING: u8 = 1;
const CELL_RETIRED: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalBranchCellAdmissionDenial {
    ForeignOwner,
    ExpiredLifecycle,
    SecondCellWhileHeld,
    ExecutingThreadReentry,
    RetirementInProgress,
    RetiredIncarnation,
    PoisonedIncarnation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalBranchCellPoisonRecovery {
    TerminallyQuarantinedPartialMutation,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SignalBranchCellCostSnapshot {
    contacts: u64,
    waits: u64,
    movements: u64,
}

impl SignalBranchCellCostSnapshot {
    pub(crate) const fn contacts(&self) -> u64 {
        self.contacts
    }

    pub(crate) const fn waits(&self) -> u64 {
        self.waits
    }

    pub(crate) const fn movements(&self) -> u64 {
        self.movements
    }
}

#[derive(Debug)]
pub(crate) struct SignalBranchExecutionCell<S> {
    state: Mutex<S>,
    fork_custody: Arc<fork_custody::SignalBranchForkCustodyGate>,
    owner_runtime_instance_id: u64,
    owner_lifecycle_identity: SignalOwnerLifecycleIdentity,
    branch_id: SignalBranchId,
    incarnation: SignalBranchCellIncarnation,
    lifecycle_posture: AtomicU8,
    counters: Arc<SignalOwnerServiceCounters>,
    contacts: AtomicU64,
    waits: AtomicU64,
    movements: AtomicU64,
    recovered_poison: AtomicBool,
    retirement_receipt: Mutex<Option<SignalBranchRetirementReceipt>>,
}

impl<S> SignalBranchExecutionCell<S> {
    pub(super) fn new(
        _construction: SignalBranchCellConstruction,
        state: S,
        owner_runtime_instance_id: u64,
        owner_lifecycle_identity: SignalOwnerLifecycleIdentity,
        branch_id: SignalBranchId,
        counters: Arc<SignalOwnerServiceCounters>,
    ) -> Self {
        Self {
            state: Mutex::new(state),
            fork_custody: Arc::new(fork_custody::SignalBranchForkCustodyGate::default()),
            owner_runtime_instance_id,
            owner_lifecycle_identity,
            branch_id,
            incarnation: SignalBranchCellIncarnation::issue(),
            lifecycle_posture: AtomicU8::new(CELL_LIVE),
            counters,
            contacts: AtomicU64::new(0),
            waits: AtomicU64::new(0),
            movements: AtomicU64::new(0),
            recovered_poison: AtomicBool::new(false),
            retirement_receipt: Mutex::new(None),
        }
    }

    pub(crate) fn branch_id(&self) -> SignalBranchId {
        self.branch_id
    }

    #[allow(
        dead_code,
        reason = "Phase 4 managed-reference admission validates this incarnation"
    )]
    pub(super) const fn incarnation(&self) -> SignalBranchCellIncarnation {
        self.incarnation
    }

    #[cfg(test)]
    pub(crate) fn with_state<R>(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        operation: impl FnOnce(&mut S, &SignalBranchCellWork<'_>) -> R,
    ) -> Result<R, SignalBranchCellAdmissionDenial> {
        self.validate_admission(admission)?;
        let _cell_hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)?;
        self.counters.record_target_cell_contact();
        self.contacts.fetch_add(1, Ordering::SeqCst);
        let mut state = self.lock_state_after_contention_observation()?;
        self.require_live_posture()?;
        let work = SignalBranchCellWork {
            counters: &self.counters,
            movements: &self.movements,
        };
        Ok(operation(&mut state, &work))
    }

    #[cfg(test)]
    pub(super) fn with_retirement<R, D>(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        operation: impl FnOnce(&mut S, &SignalBranchCellWork<'_>) -> Result<R, D>,
    ) -> Result<Result<R, D>, SignalBranchCellAdmissionDenial> {
        self.validate_admission(admission)?;
        let _cell_hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)?;
        self.counters.record_target_cell_contact();
        self.contacts.fetch_add(1, Ordering::SeqCst);
        let mut state = self.lock_state_after_contention_observation()?;
        self.lifecycle_posture
            .compare_exchange(
                CELL_LIVE,
                CELL_RETIRING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map_err(|observed| match observed {
                CELL_RETIRING => SignalBranchCellAdmissionDenial::RetirementInProgress,
                CELL_RETIRED => SignalBranchCellAdmissionDenial::RetiredIncarnation,
                _ => unreachable!("branch cell lifecycle posture is owner-defined"),
            })?;
        let mut posture = SignalBranchCellRetirementPosture {
            lifecycle_posture: &self.lifecycle_posture,
            retired: false,
        };
        let work = SignalBranchCellWork {
            counters: &self.counters,
            movements: &self.movements,
        };
        let result = operation(&mut state, &work);
        if result.is_ok() {
            self.lifecycle_posture
                .store(CELL_RETIRED, Ordering::Release);
            posture.retired = true;
        }
        Ok(result)
    }

    pub(crate) fn poison_recovery(&self) -> Option<SignalBranchCellPoisonRecovery> {
        self.recovered_poison
            .load(Ordering::Acquire)
            .then_some(SignalBranchCellPoisonRecovery::TerminallyQuarantinedPartialMutation)
    }

    pub(crate) fn cost_snapshot(&self) -> SignalBranchCellCostSnapshot {
        SignalBranchCellCostSnapshot {
            contacts: self.contacts.load(Ordering::SeqCst),
            waits: self.waits.load(Ordering::SeqCst),
            movements: self.movements.load(Ordering::SeqCst),
        }
    }

    pub(in crate::branch::owner_services) fn remains_live(&self) -> bool {
        self.lifecycle_posture.load(Ordering::Acquire) == CELL_LIVE
    }

    pub(in crate::branch::owner_services) fn take_retirement_receipt(
        &self,
    ) -> Option<SignalBranchRetirementReceipt> {
        self.retirement_receipt
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
    }

    pub(in crate::branch::owner_services) fn validate_admission(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
    ) -> Result<(), SignalBranchCellAdmissionDenial> {
        admission
            .authorize(
                self.owner_runtime_instance_id,
                self.owner_lifecycle_identity,
            )
            .map_err(SignalBranchCellAdmissionDenial::from)
    }

    fn lock_state_after_contention_observation(
        &self,
    ) -> Result<SignalBranchStateGuard<'_, S>, SignalBranchCellAdmissionDenial> {
        let custody =
            fork_custody::SignalBranchForkCustodyGate::acquire_ordinary(&self.fork_custody, || {
                self.record_cell_wait()
            });
        let state = self.lock_state_without_fork_custody()?;
        Ok(SignalBranchStateGuard {
            state,
            _custody: custody,
        })
    }

    fn lock_state_without_fork_custody(
        &self,
    ) -> Result<MutexGuard<'_, S>, SignalBranchCellAdmissionDenial> {
        match self.state.try_lock() {
            Ok(state) => Ok(state),
            Err(TryLockError::WouldBlock) => {
                self.record_cell_wait();
                match self.state.lock() {
                    Ok(state) => Ok(state),
                    Err(poisoned) => self.quarantine_poisoned_state(poisoned),
                }
            }
            Err(TryLockError::Poisoned(poisoned)) => self.quarantine_poisoned_state(poisoned),
        }
    }

    fn record_cell_wait(&self) {
        self.counters.record_target_cell_wait();
        self.waits.fetch_add(1, Ordering::SeqCst);
    }

    pub(in crate::branch::owner_services) fn acquire_fork_source_custody<'admission, 'owner>(
        self: &Arc<Self>,
        admission: &'admission SignalOwnerOperationAdmission<'owner>,
    ) -> Result<SignalBranchForkSourceCustody<'admission, 'owner, S>, SignalBranchCellAdmissionDenial>
    {
        self.validate_admission(admission)?;
        let cell_hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)?;
        Ok(fork_custody::SignalBranchForkCustodyGate::acquire_fork(
            &self.fork_custody,
            self,
            admission,
            cell_hold,
            || self.record_cell_wait(),
        ))
    }

    fn quarantine_poisoned_state<'a>(
        &self,
        poisoned: std::sync::PoisonError<MutexGuard<'a, S>>,
    ) -> Result<MutexGuard<'a, S>, SignalBranchCellAdmissionDenial> {
        self.recovered_poison.store(true, Ordering::Release);
        drop(poisoned.into_inner());
        Err(SignalBranchCellAdmissionDenial::PoisonedIncarnation)
    }

    fn require_live_posture(&self) -> Result<(), SignalBranchCellAdmissionDenial> {
        match self.lifecycle_posture.load(Ordering::Acquire) {
            CELL_LIVE => Ok(()),
            CELL_RETIRING => Err(SignalBranchCellAdmissionDenial::RetirementInProgress),
            CELL_RETIRED => Err(SignalBranchCellAdmissionDenial::RetiredIncarnation),
            _ => unreachable!("branch cell lifecycle posture is owner-defined"),
        }
    }
}

struct SignalBranchStateGuard<'a, S> {
    state: MutexGuard<'a, S>,
    _custody: fork_custody::SignalBranchOrdinaryCellCustody,
}

impl<S> std::ops::Deref for SignalBranchStateGuard<'_, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<S> std::ops::DerefMut for SignalBranchStateGuard<'_, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl From<SignalOwnerAdmissionMismatch> for SignalBranchCellAdmissionDenial {
    fn from(mismatch: SignalOwnerAdmissionMismatch) -> Self {
        match mismatch {
            SignalOwnerAdmissionMismatch::ForeignOwner => Self::ForeignOwner,
            SignalOwnerAdmissionMismatch::ExpiredLifecycle => Self::ExpiredLifecycle,
        }
    }
}

impl From<SignalOwnerBranchCellHoldDenial> for SignalBranchCellAdmissionDenial {
    fn from(denial: SignalOwnerBranchCellHoldDenial) -> Self {
        match denial {
            SignalOwnerBranchCellHoldDenial::SecondCellWhileHeld => Self::SecondCellWhileHeld,
            SignalOwnerBranchCellHoldDenial::ExecutingThreadReentry => Self::ExecutingThreadReentry,
        }
    }
}

pub(crate) struct SignalBranchCellWork<'a> {
    counters: &'a SignalOwnerServiceCounters,
    movements: &'a AtomicU64,
}

struct SignalBranchCellRetirementPosture<'a> {
    lifecycle_posture: &'a AtomicU8,
    retired: bool,
}

impl Drop for SignalBranchCellRetirementPosture<'_> {
    fn drop(&mut self) {
        if !self.retired {
            self.lifecycle_posture.store(CELL_LIVE, Ordering::Release);
        }
    }
}

impl SignalBranchCellWork<'_> {
    pub(crate) fn record_canonical_movement(&self, _permit: &SignalOwnerMovementPermit<'_>) {
        self.counters.record_canonical_movement();
        self.movements.fetch_add(1, Ordering::SeqCst);
    }

    pub(crate) fn record_retention_registry_contact(&self) {
        self.counters.record_retention_registry_contact();
    }

    pub(crate) fn record_fork_source_capture(
        &self,
        work: crate::data::graph::signal_graph::SignalGraphForkWork,
    ) {
        self.counters.record_fork_source_capture();
        self.counters
            .record_forked_mutable_graph_node_copies(work.copied_mutable_graph_nodes());
    }

    pub(crate) fn record_diagnostic_event(&self) {
        self.counters.record_diagnostic_event();
    }

    pub(crate) fn record_dropped_diagnostic_event(&self) {
        self.counters.record_dropped_diagnostic_event();
    }
}
