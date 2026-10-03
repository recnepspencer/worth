use std::collections::{BTreeMap, HashMap};
use std::sync::{Mutex, Weak};

use worth_store_physical_format::RecordArtifactFile;

use super::{PhysicalRetentionProfile, RetiredArtifact};
use crate::physical_runtime::PhysicalMutationIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalPublicationAdmissionDenial {
    ScopeConflict(PhysicalMutationIdentity),
    Growth(PhysicalRetentionGrowthDenial),
    ReclaimFenced,
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
    generations: BTreeMap<RecordArtifactFile, CandidateCharge>,
    garbage: BTreeMap<RetiredArtifact, RetainedGarbage>,
    sealed_publications: Vec<(u64, u64, u64)>,
    wal_publications: BTreeMap<u64, publication_group::RetainedWalPublication>,
    reserved_displaced_entries: u32,
    reserved_displaced_bytes: u64,
}

struct RetainedGarbage {
    bytes: u64,
    source_root: u64,
    claimed: bool,
    completed: bool,
}

struct CandidateCharge {
    bytes: u64,
    holders: u32,
    sealed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct DisplacedArtifact {
    pub(in crate::physical_runtime) source_root: u64,
    pub(in crate::physical_runtime) artifact: RetiredArtifact,
    pub(in crate::physical_runtime) bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum GarbageClaim {
    Claimed(DisplacedArtifact),
    AlreadyClaimed(DisplacedArtifact),
    Completed,
    Absent,
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
    artifact: RecordArtifactFile,
}

pub(in crate::physical_runtime) struct DisplacedCapacityLease {
    admission: Weak<PhysicalPublicationAdmission>,
    entries: u32,
    bytes: u64,
}

impl PhysicalPublicationAdmission {
    pub(in crate::physical_runtime) fn new(profile: PhysicalRetentionProfile) -> Self {
        Self {
            state: Mutex::new(AdmissionState {
                profile,
                charged_bytes: 0,
                pending: HashMap::new(),
                generations: BTreeMap::new(),
                garbage: BTreeMap::new(),
                sealed_publications: Vec::new(),
                wal_publications: BTreeMap::new(),
                reserved_displaced_entries: 0,
                reserved_displaced_bytes: 0,
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
        artifact: RecordArtifactFile,
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
        if let Some(charge) = state.generations.get_mut(&artifact) {
            charge.holders = charge.holders.saturating_add(1);
            drop(state);
            return Ok(CandidateGrowthLease {
                admission: std::sync::Arc::downgrade(self),
                artifact,
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
        state.generations.insert(
            artifact,
            CandidateCharge {
                bytes,
                holders: 1,
                sealed: false,
            },
        );
        drop(state);
        Ok(CandidateGrowthLease {
            admission: std::sync::Arc::downgrade(self),
            artifact,
        })
    }

    pub(in crate::physical_runtime) fn pending_len(&self) -> usize {
        self.lock().pending.len()
    }

    /// Reserves the entire prospective reclaim displacement roster before its
    /// first WAL effect. One extra entry is held for the active C5 publication.
    pub(in crate::physical_runtime) fn reserve_displaced_entries(
        self: &std::sync::Arc<Self>,
        entries: u32,
        bytes: u64,
    ) -> Result<DisplacedCapacityLease, PhysicalRetentionGrowthDenial> {
        let mut state = self.lock();
        let occupied = (state.garbage.len() as u32)
            .saturating_add(state.pending.len() as u32)
            .saturating_add(state.reserved_displaced_entries);
        let remaining = state.profile.growth_entries().saturating_sub(occupied);
        let required = entries.saturating_add(1);
        if entries == 0 || bytes == 0 || required > remaining || bytes > state.remaining_bytes() {
            return Err(PhysicalRetentionGrowthDenial {
                requested_bytes: bytes,
                requested_entries: required,
                remaining_bytes: state.remaining_bytes(),
                remaining_entries: remaining,
            });
        }
        state.reserved_displaced_entries += entries;
        state.reserved_displaced_bytes += bytes;
        Ok(DisplacedCapacityLease {
            admission: std::sync::Arc::downgrade(self),
            entries,
            bytes,
        })
    }

    pub(in crate::physical_runtime) fn pending_except(
        &self,
        lease: &PendingPublicationLease,
    ) -> bool {
        let Some(admission) = lease.admission.upgrade() else {
            return true;
        };
        if !std::ptr::eq(self, std::sync::Arc::as_ptr(&admission)) {
            return true;
        }
        let state = self.lock();
        !state.pending.contains_key(&lease.identity)
            || state
                .pending
                .keys()
                .any(|identity| *identity != lease.identity)
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub(in crate::physical_runtime) fn charged_growth_bytes(&self) -> u64 {
        self.lock().charged_bytes
    }

    /// Keeps the artifact generation charged after this in-flight lease ends,
    /// without invalidating another mutation's holder of the same artifact.
    pub(in crate::physical_runtime) fn seal_candidate_charge(&self, artifact: RecordArtifactFile) {
        let mut state = self.lock();
        if let Some(charge) = state.generations.get_mut(&artifact) {
            charge.sealed = true;
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn replace_profile(&self, profile: PhysicalRetentionProfile) {
        let mut state = self.lock();
        state.profile = profile;
    }

    #[cfg(test)]
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
            .saturating_sub(self.reserved_displaced_bytes)
    }

    fn remaining_entries(&self) -> u32 {
        self.profile
            .growth_entries()
            .saturating_sub(self.pending.len() as u32)
    }
}

impl CandidateGrowthLease {
    pub(in crate::physical_runtime) const fn artifact(&self) -> RecordArtifactFile {
        self.artifact
    }
}

impl PendingPublicationLease {
    /// Leaves the publication identity pending after an unresolved effect.
    ///
    /// Drop would clear the obligation. An indeterminate predecessor must keep
    /// blocking the next root change until recovery settles it.
    pub(in crate::physical_runtime) fn retain_unresolved(mut self) {
        self.admission = Weak::new();
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
        let Some(charge) = state.generations.get_mut(&self.artifact) else {
            return;
        };
        charge.holders = charge.holders.saturating_sub(1);
        if charge.holders == 0 && !charge.sealed {
            let bytes = charge.bytes;
            state.generations.remove(&self.artifact);
            state.charged_bytes = state.charged_bytes.saturating_sub(bytes);
        }
    }
}

impl Drop for DisplacedCapacityLease {
    fn drop(&mut self) {
        if let Some(admission) = self.admission.upgrade() {
            let mut state = admission.lock();
            state.reserved_displaced_entries = state
                .reserved_displaced_entries
                .checked_sub(self.entries)
                .expect("reclaim reservation is live until settlement");
            state.reserved_displaced_bytes = state
                .reserved_displaced_bytes
                .checked_sub(self.bytes)
                .expect("reclaim byte reservation is live until settlement");
        }
    }
}

#[path = "admission/garbage.rs"]
mod garbage;

#[path = "retained_bytes.rs"]
mod retained_bytes;
pub(in crate::physical_runtime) use retained_bytes::RetainedByteLease;

#[path = "publication_group.rs"]
mod publication_group;
pub(in crate::physical_runtime) use publication_group::WalPublicationReservation;

#[cfg(test)]
#[path = "admission_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "admission_unresolved.rs"]
mod admission_unresolved;
