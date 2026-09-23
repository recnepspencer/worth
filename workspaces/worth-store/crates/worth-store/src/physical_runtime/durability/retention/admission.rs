use std::collections::{BTreeMap, HashMap};
use std::sync::{Mutex, Weak};

use worth_store_physical_format::RecordArtifactFile;

use super::{PhysicalRetentionProfile, RetiredArtifact};
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
    generations: BTreeMap<RecordArtifactFile, (u64, u32)>,
    garbage: BTreeMap<RetiredArtifact, RetainedGarbage>,
    sealed_publications: Vec<(u64, u64, u64)>,
}

struct RetainedGarbage {
    bytes: u64,
    source_root: u64,
    claimed: bool,
    completed: bool,
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
        if let Some((_, holders)) = state.generations.get_mut(&artifact) {
            *holders = holders.saturating_add(1);
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
        state.generations.insert(artifact, (bytes, 1));
        drop(state);
        Ok(CandidateGrowthLease {
            admission: std::sync::Arc::downgrade(self),
            artifact,
        })
    }

    pub(in crate::physical_runtime) fn pending_len(&self) -> usize {
        self.lock().pending.len()
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub(in crate::physical_runtime) fn charged_growth_bytes(&self) -> u64 {
        self.lock().charged_bytes
    }

    /// Restores a sealed byte charge from published page identities.
    ///
    /// Reopen has no in-flight leases. The charge is the pages already named by
    /// the free-space frontier, so a new generation still reserves on top.
    pub(in crate::physical_runtime) fn reconstruct_retained_bytes(&self, bytes: u64) {
        if bytes == 0 {
            return;
        }
        let mut state = self.lock();
        state.charged_bytes = state.charged_bytes.saturating_add(bytes);
    }

    /// Keeps the artifact generation's byte charge after its in-flight lease ends.
    ///
    /// An unsettled publication keeps its candidate charged. The lease Drop
    /// becomes a no-op once the holder entry is removed here.
    pub(in crate::physical_runtime) fn seal_candidate_charge(&self, artifact: RecordArtifactFile) {
        let mut state = self.lock();
        state.generations.remove(&artifact);
    }

    pub(in crate::physical_runtime) fn replace_profile(&self, profile: PhysicalRetentionProfile) {
        let mut state = self.lock();
        state.profile = profile;
    }

    #[cfg(test)]
    pub(in crate::physical_runtime) fn remaining_growth_bytes(&self) -> u64 {
        self.lock().remaining_bytes()
    }

    pub(in crate::physical_runtime) fn retain_displaced(&self, displaced: DisplacedArtifact) {
        let mut state = self.lock();
        if state.garbage.contains_key(&displaced.artifact) {
            return;
        }
        state.charged_bytes = state.charged_bytes.saturating_add(displaced.bytes);
        state.garbage.insert(
            displaced.artifact,
            RetainedGarbage {
                bytes: displaced.bytes,
                source_root: displaced.source_root,
                claimed: false,
                completed: false,
            },
        );
    }

    pub(in crate::physical_runtime) fn next_displaced(&self) -> Option<DisplacedArtifact> {
        let state = self.lock();
        state.garbage.iter().find_map(|(artifact, garbage)| {
            (!garbage.completed).then_some(DisplacedArtifact {
                source_root: garbage.source_root,
                artifact: *artifact,
                bytes: garbage.bytes,
            })
        })
    }

    pub(in crate::physical_runtime) fn claim_displaced(
        &self,
        artifact: RetiredArtifact,
    ) -> GarbageClaim {
        let mut state = self.lock();
        let Some(garbage) = state.garbage.get_mut(&artifact) else {
            return GarbageClaim::Absent;
        };
        if garbage.completed {
            return GarbageClaim::Completed;
        }
        let displaced = DisplacedArtifact {
            source_root: garbage.source_root,
            artifact,
            bytes: garbage.bytes,
        };
        if garbage.claimed {
            return GarbageClaim::AlreadyClaimed(displaced);
        }
        garbage.claimed = true;
        GarbageClaim::Claimed(displaced)
    }

    pub(in crate::physical_runtime) fn removal_permit(
        &self,
        artifact: RetiredArtifact,
    ) -> Option<super::RetirementRemovalPermit> {
        let state = self.lock();
        let garbage = state.garbage.get(&artifact)?;
        (garbage.claimed && !garbage.completed)
            .then_some(super::RetirementRemovalPermit::issued(artifact))
    }

    pub(in crate::physical_runtime) fn revert_displaced_claim(&self, artifact: RetiredArtifact) {
        let mut state = self.lock();
        if let Some(garbage) = state.garbage.get_mut(&artifact) {
            if !garbage.completed {
                garbage.claimed = false;
            }
        }
    }

    pub(in crate::physical_runtime) fn complete_displaced(&self, artifact: RetiredArtifact) {
        let mut state = self.lock();
        let Some(garbage) = state.garbage.get_mut(&artifact) else {
            return;
        };
        if garbage.completed {
            return;
        }
        let bytes = garbage.bytes;
        garbage.completed = true;
        garbage.claimed = true;
        state.charged_bytes = state.charged_bytes.saturating_sub(bytes);
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
        let Some((bytes, holders)) = state.generations.get_mut(&self.artifact) else {
            return;
        };
        *holders = holders.saturating_sub(1);
        if *holders == 0 {
            let bytes = *bytes;
            state.generations.remove(&self.artifact);
            state.charged_bytes = state.charged_bytes.saturating_sub(bytes);
        }
    }
}

#[path = "retained_bytes.rs"]
mod retained_bytes;

#[cfg(test)]
#[path = "admission_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "admission_unresolved.rs"]
mod admission_unresolved;
