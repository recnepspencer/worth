//! Reversible initial-owner transition, with no operation admission.
use super::{SignalOwnerLifecycleState, OWNER_CLOSING, OWNER_OPEN};
use crate::branch::SignalOwnerStopDenial as Denial;
use std::sync::{atomic::Ordering, MutexGuard};

pub(in crate::branch::owner_services) struct InitialEmptyStopAdmission<'a> {
    lifecycle: &'a SignalOwnerLifecycleState,
    _gate: MutexGuard<'a, ()>,
}
impl SignalOwnerLifecycleState {
    pub(in crate::branch::owner_services) fn try_initial_empty_stop_admission(
        &self,
    ) -> Result<InitialEmptyStopAdmission<'_>, Denial> {
        let gate = self.transition_gate.try_lock().map_err(|e| match e {
            std::sync::TryLockError::WouldBlock => Denial::Busy,
            std::sync::TryLockError::Poisoned(_) => Denial::Poisoned,
        })?;
        let state = self.phase_and_count.load(Ordering::Acquire);
        if state != OWNER_OPEN {
            return Err(if state < OWNER_CLOSING {
                Denial::Busy
            } else {
                Denial::OwnerUnavailable
            });
        }
        if self.cleanup_claimed.load(Ordering::Acquire) {
            return Err(Denial::OwnerUnavailable);
        }
        Ok(InitialEmptyStopAdmission {
            lifecycle: self,
            _gate: gate,
        })
    }
    pub(in crate::branch::owner_services) fn try_resume_initial_empty_stop(
        &self,
    ) -> Result<(), Denial> {
        let _gate = self.transition_gate.try_lock().map_err(|_| Denial::Busy)?;
        if self.cleanup_claimed.load(Ordering::Acquire) {
            return Err(Denial::OwnerUnavailable);
        }
        self.phase_and_count
            .compare_exchange(
                OWNER_CLOSING,
                OWNER_OPEN,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|_| Denial::OwnerUnavailable)
    }
    pub(in crate::branch::owner_services) fn try_prepared_initial_empty_stop_guard(
        &self,
    ) -> Result<impl Drop + '_, Denial> {
        let gate = self.transition_gate.try_lock().map_err(|_| Denial::Busy)?;
        if self.phase_and_count.load(Ordering::Acquire) != OWNER_CLOSING
            || self.cleanup_claimed.load(Ordering::Acquire)
        {
            return Err(Denial::OwnerUnavailable);
        }
        Ok(gate)
    }
}
impl InitialEmptyStopAdmission<'_> {
    pub(in crate::branch::owner_services) fn begin(self) -> Result<(), Denial> {
        self.lifecycle
            .phase_and_count
            .compare_exchange(
                OWNER_OPEN,
                OWNER_CLOSING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|_| Denial::Busy)
    }
}

/// Holds the real transition gate until the composed owner commits or rolls back.
pub(crate) struct SignalPreparedOwnerStopGate<'a> {
    lifecycle: &'a SignalOwnerLifecycleState,
    _gate: MutexGuard<'a, ()>,
    committed: bool,
}
impl SignalOwnerLifecycleState {
    pub(crate) fn try_authored_stop_gate(&self) -> Result<SignalPreparedOwnerStopGate<'_>, Denial> {
        let admission = self.try_initial_empty_stop_admission()?;
        admission
            .lifecycle
            .phase_and_count
            .compare_exchange(
                OWNER_OPEN,
                OWNER_CLOSING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map_err(|_| Denial::Busy)?;
        Ok(SignalPreparedOwnerStopGate {
            lifecycle: admission.lifecycle,
            _gate: admission._gate,
            committed: false,
        })
    }
}
impl SignalPreparedOwnerStopGate<'_> {
    pub(crate) fn commit(mut self) {
        self.committed = true;
    }
}
impl Drop for SignalPreparedOwnerStopGate<'_> {
    fn drop(&mut self) {
        if !self.committed {
            // This guard still owns transition_gate, so no cleanup or phase transition can race rollback.
            self.lifecycle
                .phase_and_count
                .store(OWNER_OPEN, Ordering::Release);
        }
    }
}
