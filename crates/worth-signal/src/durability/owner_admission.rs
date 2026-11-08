//! Exclusive live owner custody, shared by every reader of one durable medium.
use super::{SignalDurableHomeBinding, SignalDurableMediumDenial as D};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{DurableSignalOwnerIdentity, HomeIncarnation};

/// Names the actual process-local runtime holding a durable owner's admission.
/// This observation is neither serializable identity nor admission authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SignalLiveOwnerIdentity {
    owner: DurableSignalOwnerIdentity,
    incarnation: HomeIncarnation,
    runtime_instance: u64,
}
impl SignalLiveOwnerIdentity {
    pub const fn owner(self) -> DurableSignalOwnerIdentity {
        self.owner
    }
    pub const fn incarnation(self) -> HomeIncarnation {
        self.incarnation
    }
    pub const fn runtime_instance(self) -> u64 {
        self.runtime_instance
    }
}

#[derive(Clone, Copy)]
enum AdmissionPosture {
    Available,
    Claimed(SignalLiveOwnerIdentity),
    Retired,
}
struct AdmissionState {
    owner: DurableSignalOwnerIdentity,
    incarnation: HomeIncarnation,
    posture: Mutex<AdmissionPosture>,
    _memory: Option<ExecutionMemoryReservation>,
}
/// One stable registry belongs to the actual medium, never to a readmission call.
pub struct SignalDurableOwnerAdmissionRegistry {
    state: Arc<AdmissionState>,
}
impl std::fmt::Debug for SignalDurableOwnerAdmissionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalDurableOwnerAdmissionRegistry")
            .field("owner", &self.state.owner)
            .field("incarnation", &self.state.incarnation)
            .finish_non_exhaustive()
    }
}
impl SignalDurableOwnerAdmissionRegistry {
    pub fn new(
        binding: &SignalDurableHomeBinding,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, D> {
        let charge = crate::data::retained_storage::arc_allocation_charge::<AdmissionState>()
            .map_err(|_| D::OwnerAdmissionCapacity)?;
        let memory = execution
            .reserve_retained_memory(charge.bytes())
            .map_err(D::Request)?;
        Ok(Self {
            state: Arc::new(AdmissionState {
                owner: binding.owner(),
                incarnation: binding.incarnation(),
                posture: Mutex::new(AdmissionPosture::Available),
                _memory: memory,
            }),
        })
    }
    fn posture(&self) -> Result<MutexGuard<'_, AdmissionPosture>, D> {
        self.state.posture.try_lock().map_err(|error| match error {
            TryLockError::WouldBlock => D::OwnerAdmissionBusy,
            TryLockError::Poisoned(_) => D::OwnerAdmissionPoisoned,
        })
    }
    pub(crate) fn admit(
        &self,
        binding: &SignalDurableHomeBinding,
        runtime_instance: u64,
    ) -> Result<SignalDurableOwnerAdmission, D> {
        if binding.owner() != self.state.owner {
            return Err(D::ForeignOwner);
        }
        self.state
            .incarnation
            .fence()
            .admit(binding.incarnation().fence())
            .map_err(D::HomeIncarnation)?;
        let mut posture = self.posture()?;
        match *posture {
            AdmissionPosture::Available => {
                let identity = SignalLiveOwnerIdentity {
                    owner: self.state.owner,
                    incarnation: self.state.incarnation,
                    runtime_instance,
                };
                *posture = AdmissionPosture::Claimed(identity);
                Ok(worth_proof::SignalOwnerReadmitted::from_owner(
                    SignalDurableOwnerClaim {
                        state: Arc::clone(&self.state),
                        identity,
                    },
                ))
            }
            AdmissionPosture::Claimed(holder) => Err(D::OwnerAlreadyReadmitted(holder)),
            AdmissionPosture::Retired => Err(D::Unavailable),
        }
    }
    /// Irreversibly stop new admission before releasing the medium's serving share.
    /// A live owner's actual custody refuses retirement and names that holder.
    pub fn retire(&self) -> Result<(), D> {
        let mut posture = self.posture()?;
        match *posture {
            AdmissionPosture::Available => {
                *posture = AdmissionPosture::Retired;
                Ok(())
            }
            AdmissionPosture::Claimed(holder) => Err(D::OwnerAlreadyReadmitted(holder)),
            AdmissionPosture::Retired => Ok(()),
        }
    }
}

pub(crate) type SignalDurableOwnerAdmission =
    worth_proof::SignalOwnerReadmitted<SignalDurableOwnerClaim>;
pub(crate) struct SignalDurableOwnerClaim {
    state: Arc<AdmissionState>,
    identity: SignalLiveOwnerIdentity,
}
impl worth_proof::AuthorityMarker for SignalDurableOwnerClaim {}
mod no_effect_original;
pub use no_effect_original::SignalNoEffectOriginal;
impl std::fmt::Debug for SignalDurableOwnerClaim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.identity, f)
    }
}
impl Drop for SignalDurableOwnerClaim {
    fn drop(&mut self) {
        let mut posture = self
            .state
            .posture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match *posture {
            AdmissionPosture::Claimed(holder) => {
                assert_eq!(
                    holder, self.identity,
                    "only the original claim can release its owner"
                );
                *posture = AdmissionPosture::Available;
            }
            AdmissionPosture::Available | AdmissionPosture::Retired => {
                panic!("live Signal admission was released without its original custody")
            }
        }
    }
}
