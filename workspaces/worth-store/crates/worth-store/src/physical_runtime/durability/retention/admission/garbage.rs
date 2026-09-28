use super::{DisplacedArtifact, GarbageClaim, PhysicalPublicationAdmission, RetainedGarbage};
use crate::physical_runtime::durability::retention::RetiredArtifact;

impl PhysicalPublicationAdmission {
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

    /// Admission for newly obsolete whole arenas, whose capacity was live
    /// payload rather than already charged rewrite garbage.
    pub(in crate::physical_runtime) fn admit_displaced_arena(
        &self,
        displaced: DisplacedArtifact,
    ) -> bool {
        if !matches!(displaced.artifact, RetiredArtifact::Arena { .. }) {
            return false;
        }
        let mut state = self.lock();
        if state.garbage.contains_key(&displaced.artifact) {
            return true;
        }
        if displaced.bytes > state.remaining_bytes()
            || state.garbage.len().saturating_add(state.pending.len())
                >= state.profile.growth_entries() as usize
        {
            return false;
        }
        state.charged_bytes += displaced.bytes;
        state.garbage.insert(
            displaced.artifact,
            RetainedGarbage {
                bytes: displaced.bytes,
                source_root: displaced.source_root,
                claimed: false,
                completed: false,
            },
        );
        true
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

    /// Reinstates only the exact garbage claim named by an authenticated
    /// unresolved retirement intent. A mismatched replay never changes claim state.
    pub(in crate::physical_runtime) fn claim_recovered_displaced_exact(
        &self,
        displaced: DisplacedArtifact,
    ) -> bool {
        let mut state = self.lock();
        let Some(garbage) = state.garbage.get_mut(&displaced.artifact) else {
            return false;
        };
        if garbage.completed
            || garbage.source_root != displaced.source_root
            || garbage.bytes != displaced.bytes
        {
            return false;
        }
        garbage.claimed = true;
        true
    }

    pub(in crate::physical_runtime) fn removal_permit(
        &self,
        artifact: RetiredArtifact,
    ) -> Option<super::super::RetirementRemovalPermit> {
        let state = self.lock();
        let garbage = state.garbage.get(&artifact)?;
        (garbage.claimed && !garbage.completed)
            .then_some(super::super::RetirementRemovalPermit::issued(artifact))
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
}
