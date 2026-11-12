//! The original admission remains retained while stopped ownership cannot be readmitted.
use super::{AdmissionPosture, SignalDurableOwnerClaim, SignalLiveOwnerIdentity};
use crate::branch::owner_services::lifecycle_state::SignalStoppedOwnerGuard;
use std::sync::{MutexGuard, TryLockError};
pub(crate) enum SignalDurableAdmissionRetirementDenial {
    Busy,
    Poisoned,
}
pub(crate) struct SignalPreparedDurableAdmissionRetirement<'a> {
    posture: MutexGuard<'a, AdmissionPosture>,
    identity: SignalLiveOwnerIdentity,
}
impl SignalDurableOwnerClaim {
    pub(crate) fn prepare_retirement<'a>(
        &'a self,
        stopped: &'a SignalStoppedOwnerGuard<'_>,
    ) -> Result<SignalPreparedDurableAdmissionRetirement<'a>, SignalDurableAdmissionRetirementDenial>
    {
        assert_eq!(
            stopped.owner_runtime_instance(),
            self.identity.runtime_instance,
            "the stop guard belongs to this actual owner admission"
        );
        let posture = self.state.posture.try_lock().map_err(|cause| match cause {
            TryLockError::WouldBlock => SignalDurableAdmissionRetirementDenial::Busy,
            TryLockError::Poisoned(_) => SignalDurableAdmissionRetirementDenial::Poisoned,
        })?;
        match *posture {
            AdmissionPosture::Claimed(holder) | AdmissionPosture::RetiredClaimed(holder) => {
                assert_eq!(
                    holder, self.identity,
                    "only the original claim fences its owner"
                )
            }
            AdmissionPosture::Available | AdmissionPosture::Retired => {
                panic!("an actual retained claim cannot be absent from its registry")
            }
        }
        Ok(SignalPreparedDurableAdmissionRetirement {
            posture,
            identity: self.identity,
        })
    }
}
impl SignalPreparedDurableAdmissionRetirement<'_> {
    /// The owner invokes this after its same stop closes retention, with no failable step.
    pub(crate) fn commit(mut self) {
        *self.posture = AdmissionPosture::RetiredClaimed(self.identity);
    }
}
