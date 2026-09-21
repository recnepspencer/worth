use std::collections::{BTreeMap, HashMap};
use std::sync::{Mutex, Weak};

use super::PhysicalRetentionProfile;
use crate::physical_runtime::PhysicalMutationIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalPublicationAdmissionDenial {
    ScopeConflict(PhysicalMutationIdentity),
    Growth(PhysicalRetentionGrowthDenial),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PhysicalRetentionGrowthDenial {
    pub requested_bytes: u64,
    pub requested_entries: u32,
    pub remaining_bytes: u64,
    pub remaining_entries: u32,
}

struct AdmissionState {
    profile: PhysicalRetentionProfile,
    charged_bytes: u64,
    pending: HashMap<PhysicalMutationIdentity, u32>,
    generations: BTreeMap<u64, (u64, u32)>,
}

pub(in crate::physical_runtime) struct PhysicalPublicationAdmission {
    state: Mutex<AdmissionState>,
}

pub(in crate::physical_runtime) struct PendingPublicationLease {
    admission: Weak<PhysicalPublicationAdmission>,
    identity: PhysicalMutationIdentity,
}

pub(in crate::physical_runtime) struct CandidateGrowthLease {
    admission: Weak<PhysicalPublicationAdmission>,
    generation: u64,
}

impl PhysicalPublicationAdmission {
    pub(in crate::physical_runtime) fn new(profile: PhysicalRetentionProfile) -> Self {
        Self {
            state: Mutex::new(AdmissionState {
                profile,
                charged_bytes: 0,
                pending: HashMap::new(),
                generations: BTreeMap::new(),
            }),
        }
    }

    /// Registers one root-changing publication only when no other publication is pending.
    ///
    /// A conflict returns the blocker and takes no lease, so the earlier publication
    /// can still settle.
    pub(in crate::physical_runtime) fn register_exclusive_pending(
        self: &std::sync::Arc<Self>,
        identity: PhysicalMutationIdentity,
    ) -> Result<PendingPublicationLease, PhysicalPublicationAdmissionDenial> {
        let mut state = self.lock();
        if let Some(blocker) = state
            .pending
            .keys()
            .copied()
            .find(|pending| *pending != identity)
        {
            return Err(PhysicalPublicationAdmissionDenial::ScopeConflict(blocker));
        }
        if let Some(holders) = state.pending.get_mut(&identity) {
            *holders = holders.saturating_add(1);
            drop(state);
            return Ok(PendingPublicationLease {
                admission: std::sync::Arc::downgrade(self),
                identity,
            });
        }
        let remaining_entries = state
            .profile
            .growth_entries()
            .saturating_sub(state.pending.len() as u32);
        if remaining_entries == 0 {
            return Err(PhysicalPublicationAdmissionDenial::Growth(
                PhysicalRetentionGrowthDenial {
                    requested_bytes: 0,
                    requested_entries: 1,
                    remaining_bytes: state.remaining_bytes(),
                    remaining_entries: 0,
                },
            ));
        }
        state.pending.insert(identity, 1);
        drop(state);
        Ok(PendingPublicationLease {
            admission: std::sync::Arc::downgrade(self),
            identity,
        })
    }

    pub(in crate::physical_runtime) fn reserve_candidate(
        self: &std::sync::Arc<Self>,
        generation: u64,
        bytes: u64,
    ) -> Result<CandidateGrowthLease, PhysicalRetentionGrowthDenial> {
        let mut state = self.lock();
        if bytes == 0 {
            return Err(PhysicalRetentionGrowthDenial {
                requested_bytes: 0,
                requested_entries: 0,
                remaining_bytes: state.remaining_bytes(),
                remaining_entries: state.remaining_entries(),
            });
        }
        if let Some((_, holders)) = state.generations.get_mut(&generation) {
            *holders = holders.saturating_add(1);
            drop(state);
            return Ok(CandidateGrowthLease {
                admission: std::sync::Arc::downgrade(self),
                generation,
            });
        }
        let remaining = state.remaining_bytes();
        if bytes > remaining {
            return Err(PhysicalRetentionGrowthDenial {
                requested_bytes: bytes,
                requested_entries: 0,
                remaining_bytes: remaining,
                remaining_entries: state.remaining_entries(),
            });
        }
        state.charged_bytes = state.charged_bytes.saturating_add(bytes);
        state.generations.insert(generation, (bytes, 1));
        drop(state);
        Ok(CandidateGrowthLease {
            admission: std::sync::Arc::downgrade(self),
            generation,
        })
    }

    pub(in crate::physical_runtime) fn pending_len(&self) -> usize {
        self.lock().pending.len()
    }

    /// Keeps the generation's byte charge after its in-flight lease ends.
    ///
    /// Publication success converts a candidate into retained growth. The lease
    /// Drop becomes a no-op once the holder entry is removed here.
    pub(in crate::physical_runtime) fn seal_candidate_charge(&self, generation: u64) {
        let mut state = self.lock();
        state.generations.remove(&generation);
    }

    pub(in crate::physical_runtime) fn replace_profile(&self, profile: PhysicalRetentionProfile) {
        let mut state = self.lock();
        state.profile = profile;
    }

    pub(in crate::physical_runtime) fn remaining_growth_bytes(&self) -> u64 {
        self.lock().remaining_bytes()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, AdmissionState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl AdmissionState {
    fn remaining_bytes(&self) -> u64 {
        self.profile
            .growth_bytes()
            .saturating_sub(self.charged_bytes)
    }

    fn remaining_entries(&self) -> u32 {
        self.profile
            .growth_entries()
            .saturating_sub(self.pending.len() as u32)
    }
}

impl CandidateGrowthLease {
    pub(in crate::physical_runtime) const fn generation(&self) -> u64 {
        self.generation
    }
}

impl Drop for PendingPublicationLease {
    fn drop(&mut self) {
        let Some(admission) = self.admission.upgrade() else {
            return;
        };
        let mut state = admission.lock();
        let Some(holders) = state.pending.get_mut(&self.identity) else {
            return;
        };
        *holders = holders.saturating_sub(1);
        if *holders == 0 {
            state.pending.remove(&self.identity);
        }
    }
}

impl Drop for CandidateGrowthLease {
    fn drop(&mut self) {
        let Some(admission) = self.admission.upgrade() else {
            return;
        };
        let mut state = admission.lock();
        let Some((bytes, holders)) = state.generations.get_mut(&self.generation) else {
            return;
        };
        *holders = holders.saturating_sub(1);
        if *holders == 0 {
            let bytes = *bytes;
            state.generations.remove(&self.generation);
            state.charged_bytes = state.charged_bytes.saturating_sub(bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growth_cannot_consume_progress_headroom() {
        let profile = PhysicalRetentionProfile::new(100, 4, 40, 1).unwrap();
        let admission = std::sync::Arc::new(PhysicalPublicationAdmission::new(profile));
        let first = admission.reserve_candidate(1, 60).unwrap();
        let Err(denied) = admission.reserve_candidate(2, 1) else {
            panic!("a second generation cannot consume progress headroom");
        };
        assert_eq!(denied.remaining_bytes, 0);
        assert_eq!(denied.requested_bytes, 1);
        drop(first);
        let shared = admission.reserve_candidate(7, 60).unwrap();
        let again = admission.reserve_candidate(7, 60).unwrap();
        assert_eq!(admission.lock().charged_bytes, 60);
        drop(shared);
        assert_eq!(admission.lock().charged_bytes, 60);
        drop(again);
        assert_eq!(admission.lock().charged_bytes, 0);
        assert!(admission.reserve_candidate(3, 61).is_err());
    }

    #[test]
    fn sealed_candidate_charge_survives_lease_drop() {
        let profile = PhysicalRetentionProfile::new(100, 4, 40, 1).unwrap();
        let admission = std::sync::Arc::new(PhysicalPublicationAdmission::new(profile));
        let lease = admission.reserve_candidate(9, 60).unwrap();
        admission.seal_candidate_charge(lease.generation());
        drop(lease);
        assert_eq!(admission.remaining_growth_bytes(), 0);
        let Err(denied) = admission.reserve_candidate(10, 1) else {
            panic!("sealed retained growth must keep denying one-over requests");
        };
        assert_eq!(denied.remaining_bytes, 0);
        assert_eq!(denied.requested_bytes, 1);
    }

}