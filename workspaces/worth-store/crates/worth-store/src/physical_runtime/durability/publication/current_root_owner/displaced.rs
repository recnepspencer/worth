use super::{PhysicalCurrentRootOwner, PhysicalCurrentRootState};
use crate::physical_runtime::durability::retention::{DisplacedSegment, GarbageClaim};
use crate::physical_runtime::PhysicalRetirementDenial;

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn note_displaced_segment(
        &self,
        source_root: u64,
        segment_id: u64,
        generation: u64,
        bytes: u64,
    ) {
        *self
            .displaced
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(
            crate::physical_runtime::durability::retention::DisplacedSegment {
                source_root,
                segment_id,
                generation,
                bytes,
            },
        );
    }

    pub(in crate::physical_runtime) fn release_rewrite_candidate(&self) {
        self.rewrite_growth
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        self.displaced
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
    }

    pub(in crate::physical_runtime) fn commit_rewrite_candidate(&self) {
        if let Some(displaced) = self
            .displaced
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            self.publication.retain_displaced(displaced);
        }
        // The published generation is live payload. Dropping the lease releases
        // it from the excess-obsolete budget. The displaced source, if any, was
        // charged above and stays until reclaim.
        drop(std::mem::take(
            &mut *self
                .rewrite_growth
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        ));
    }

    /// Keeps a reserved candidate charged when publication did not settle.
    ///
    /// The displaced-source note is discarded: the current root did not advance,
    /// so the source is not garbage.
    pub(in crate::physical_runtime) fn retain_unresolved_rewrite_candidate(&self) {
        let leases = std::mem::take(
            &mut *self
                .rewrite_growth
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        for lease in leases {
            self.publication.seal_candidate_charge(lease.generation());
        }
        self.displaced
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
    }

    /// Claims the next displaced segment only while no protected root still
    /// names it and the published tail has already moved on.
    ///
    /// The publication-state lock is the same lock a new reader takes before
    /// registration, so a capture cannot land between this check and the claim.
    pub(in crate::physical_runtime) fn claim_retirement(
        &self,
    ) -> Result<Option<DisplacedSegment>, PhysicalRetirementDenial> {
        let state = self.lock_publication_state();
        let Some(displaced) = self.publication.next_displaced() else {
            return Err(PhysicalRetirementDenial::Absent);
        };
        if let Some(denial) = retirement_blocked(self, &state, &displaced) {
            return Err(denial);
        }
        match self
            .publication
            .claim_displaced(displaced.segment_id, displaced.generation)
        {
            GarbageClaim::Completed => Ok(None),
            GarbageClaim::Absent => Err(PhysicalRetirementDenial::Absent),
            GarbageClaim::Claimed(_) | GarbageClaim::AlreadyClaimed(_) => Ok(Some(displaced)),
        }
    }

    pub(in crate::physical_runtime) fn blocked_retirement(
        &self,
        displaced: &DisplacedSegment,
    ) -> Option<PhysicalRetirementDenial> {
        let state = self.lock_publication_state();
        retirement_blocked(self, &state, displaced)
    }

    pub(in crate::physical_runtime) fn revert_displaced_claim(
        &self,
        segment_id: u64,
        generation: u64,
    ) {
        self.publication
            .revert_displaced_claim(segment_id, generation);
    }

    pub(in crate::physical_runtime) fn complete_displaced(&self, segment_id: u64, generation: u64) {
        self.publication.complete_displaced(segment_id, generation);
    }

    pub(in crate::physical_runtime) fn removal_permit(
        &self,
        segment_id: u64,
        generation: u64,
    ) -> Option<crate::physical_runtime::durability::RetirementRemovalPermit> {
        self.publication.removal_permit(segment_id, generation)
    }

    pub(in crate::physical_runtime) fn next_displaced_segment(&self) -> Option<DisplacedSegment> {
        self.publication.next_displaced()
    }
}

fn retirement_blocked(
    owner: &PhysicalCurrentRootOwner,
    state: &PhysicalCurrentRootState,
    displaced: &DisplacedSegment,
) -> Option<PhysicalRetirementDenial> {
    if owner.read_protection.protects_root(displaced.source_root)
        || owner
            .read_protection
            .protects_inline_segment(displaced.segment_id, displaced.generation)
    {
        return Some(PhysicalRetirementDenial::Protected);
    }
    let still_published = state
        .current_root
        .last_inline_segment()
        .is_some_and(|cell| {
            cell.segment_id().get() == displaced.segment_id
                && cell.generation().get() == displaced.generation
        });
    if still_published {
        return Some(PhysicalRetirementDenial::Retained);
    }
    if owner.publication.pending_len() > 0 {
        return Some(PhysicalRetirementDenial::Unresolved);
    }
    None
}
