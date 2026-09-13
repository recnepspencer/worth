use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use crate::state::SignalBranchId;

use super::basis::{
    admit_signal_branch_observation, AdmittedSignalBranchBasis, AdmittedSignalBranchBasisInner,
};
use super::{
    SignalBranchAdmissionLease, SignalBranchObservation, SignalBranchRetentionAcquisitionDenial,
};

#[path = "basis_registry/acquisition.rs"]
mod acquisition;
#[path = "basis_registry/currentness.rs"]
mod currentness;
#[path = "basis_registry/direct.rs"]
mod direct;
#[path = "basis_registry/key.rs"]
mod key;
#[path = "basis_registry/lazy.rs"]
mod lazy;
#[cfg(test)]
#[path = "basis_registry/tests.rs"]
mod tests;

use acquisition::{AcquiringEntry, RegistryEntry, SingleFlightCompletion};
use key::SignalBranchBasisRegistryKey;

#[derive(Debug)]
struct SignalBranchBasisRegistryState {
    next_reservation_id: u64,
    next_registration_id: u64,
    entries: HashMap<SignalBranchBasisRegistryKey, RegistryEntry>,
}

/// Weak cleanup lease installed in one admitted basis. The registry owns only
/// weak entries, so canonicalization cannot keep an observation, owner, or
/// component lease alive by itself.
#[derive(Debug)]
pub(crate) struct SignalBranchBasisRegistryLease {
    state: Weak<Mutex<SignalBranchBasisRegistryState>>,
    key: SignalBranchBasisRegistryKey,
    registration_id: u64,
}

/// Signal-owner-local exact-basis canonicalization. Lazy admissions use a
/// single-flight reservation so exactly one claimant contacts the retention
/// owner for a given live basis key.
#[derive(Debug, Clone)]
pub(crate) struct SignalBranchBasisRegistry {
    state: Arc<Mutex<SignalBranchBasisRegistryState>>,
}

enum AdmissionDecision {
    Ready(Arc<AdmittedSignalBranchBasisInner>),
    Join(Arc<SingleFlightCompletion>),
    Claim {
        reservation_id: u64,
        completion: Arc<SingleFlightCompletion>,
    },
    OwnerReentry,
}

impl SignalBranchBasisRegistry {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SignalBranchBasisRegistryState {
                next_reservation_id: 0,
                next_registration_id: 0,
                entries: HashMap::new(),
            })),
        }
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, SignalBranchBasisRegistryState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn begin_admission(
    state: &mut SignalBranchBasisRegistryState,
    key: &SignalBranchBasisRegistryKey,
) -> Result<AdmissionDecision, SignalBranchRetentionAcquisitionDenial> {
    if let Some(entry) = state.entries.get(key) {
        match entry {
            RegistryEntry::Ready { basis, .. } => {
                if let Some(existing) = basis.upgrade() {
                    return Ok(AdmissionDecision::Ready(existing));
                }
                state.entries.remove(key);
            }
            RegistryEntry::Acquiring(acquiring) => {
                if acquiring.initiating_thread == std::thread::current().id() {
                    return Ok(AdmissionDecision::OwnerReentry);
                }
                acquiring.completion.record_joiner();
                return Ok(AdmissionDecision::Join(Arc::clone(&acquiring.completion)));
            }
        }
    }
    let reservation_id = state
        .next_reservation_id
        .checked_add(1)
        .ok_or(SignalBranchRetentionAcquisitionDenial::IdentityExhausted)?;
    state.next_reservation_id = reservation_id;
    let completion = Arc::new(SingleFlightCompletion::new());
    state.entries.insert(
        key.clone(),
        RegistryEntry::Acquiring(AcquiringEntry {
            reservation_id,
            initiating_thread: std::thread::current().id(),
            completion: Arc::clone(&completion),
        }),
    );
    Ok(AdmissionDecision::Claim {
        reservation_id,
        completion,
    })
}

#[cfg(test)]
fn test_completion(
    registry: &SignalBranchBasisRegistry,
    key: &SignalBranchBasisRegistryKey,
) -> Option<Arc<SingleFlightCompletion>> {
    let state = registry.lock_state();
    match state.entries.get(key) {
        Some(RegistryEntry::Acquiring(acquiring)) => Some(Arc::clone(&acquiring.completion)),
        _ => None,
    }
}

fn live_ready_basis(
    state: &mut SignalBranchBasisRegistryState,
    key: &SignalBranchBasisRegistryKey,
) -> Option<Arc<AdmittedSignalBranchBasisInner>> {
    let Some(RegistryEntry::Ready { basis, .. }) = state.entries.get(key) else {
        return None;
    };
    match basis.upgrade() {
        Some(existing) => Some(existing),
        None => {
            state.entries.remove(key);
            None
        }
    }
}

fn new_basis(
    state: &mut SignalBranchBasisRegistryState,
    registry_state: &Arc<Mutex<SignalBranchBasisRegistryState>>,
    key: SignalBranchBasisRegistryKey,
    observation: SignalBranchObservation,
    branch_id: SignalBranchId,
    retention: SignalBranchAdmissionLease,
) -> Result<AdmittedSignalBranchBasis, SignalBranchRetentionAcquisitionDenial> {
    let candidate = admit_signal_branch_observation(observation, branch_id, retention);
    let registration_id = state
        .next_registration_id
        .checked_add(1)
        .ok_or(SignalBranchRetentionAcquisitionDenial::IdentityExhausted)?;
    state.next_registration_id = registration_id;
    let lease = SignalBranchBasisRegistryLease {
        state: Arc::downgrade(registry_state),
        key: key.clone(),
        registration_id,
    };
    candidate
        .0
        .registry_lease
        .set(lease)
        .expect("a freshly issued Signal basis has one registry lease slot");
    let replaced = state.entries.insert(
        key,
        RegistryEntry::Ready {
            registration_id,
            basis: Arc::downgrade(&candidate.0),
        },
    );
    assert!(
        replaced.is_none() || matches!(replaced, Some(RegistryEntry::Acquiring(_))),
        "a ready Signal basis may replace only its own reservation"
    );
    Ok(candidate)
}

fn finish_denied(
    registry_state: &Arc<Mutex<SignalBranchBasisRegistryState>>,
    key: &SignalBranchBasisRegistryKey,
    reservation_id: u64,
    completion: &SingleFlightCompletion,
    denial: SignalBranchRetentionAcquisitionDenial,
) {
    let owns_reservation = {
        let mut state = registry_state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if matches!(
            state.entries.get(key),
            Some(RegistryEntry::Acquiring(AcquiringEntry {
                reservation_id: current,
                ..
            })) if *current == reservation_id
        ) {
            state.entries.remove(key);
            true
        } else {
            false
        }
    };
    if owns_reservation {
        completion.finish(Err(denial));
    }
}

struct AcquisitionClaimGuard {
    state: Weak<Mutex<SignalBranchBasisRegistryState>>,
    key: SignalBranchBasisRegistryKey,
    reservation_id: u64,
    completion: Arc<SingleFlightCompletion>,
    armed: bool,
}

impl AcquisitionClaimGuard {
    fn new(
        state: &Arc<Mutex<SignalBranchBasisRegistryState>>,
        key: SignalBranchBasisRegistryKey,
        reservation_id: u64,
        completion: Arc<SingleFlightCompletion>,
    ) -> Self {
        Self {
            state: Arc::downgrade(state),
            key,
            reservation_id,
            completion,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for AcquisitionClaimGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if let Some(state) = self.state.upgrade() {
            finish_denied(
                &state,
                &self.key,
                self.reservation_id,
                &self.completion,
                SignalBranchRetentionAcquisitionDenial::OwnerOperationPanicked,
            );
        } else {
            self.completion.finish(Err(
                SignalBranchRetentionAcquisitionDenial::OwnerOperationPanicked,
            ));
        }
    }
}

impl Default for SignalBranchBasisRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SignalBranchBasisRegistryLease {
    fn drop(&mut self) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        let mut state = state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if matches!(
            state.entries.get(&self.key),
            Some(RegistryEntry::Ready {
                registration_id,
                ..
            }) if *registration_id == self.registration_id
        ) {
            state.entries.remove(&self.key);
        }
    }
}
