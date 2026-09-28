use super::{PhysicalCurrentRootOwner, PhysicalCurrentRootState};
use crate::physical_runtime::durability::retention::{
    DisplacedArtifact, GarbageClaim, RetiredArtifact,
};
use crate::physical_runtime::PhysicalRetirementDenial;

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn note_displaced(
        &self,
        source_root: u64,
        artifact: RetiredArtifact,
        bytes: u64,
    ) {
        *self
            .displaced
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(DisplacedArtifact {
            source_root,
            artifact,
            bytes,
        });
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
            self.publication.seal_candidate_charge(lease.artifact());
        }
        self.displaced
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
    }

    /// Claims the next displaced generation only while no protected root still
    /// names it and the published tail has already moved on.
    ///
    /// The publication-state lock is the same lock a new reader takes before
    /// registration, so a capture cannot land between this check and the claim.
    pub(in crate::physical_runtime) fn claim_retirement(
        &self,
    ) -> Result<Option<DisplacedArtifact>, PhysicalRetirementDenial> {
        let state = self.lock_publication_state();
        let Some(displaced) = self.publication.next_displaced() else {
            return Err(PhysicalRetirementDenial::Absent);
        };
        if let Some(denial) = retirement_blocked(self, &state, &displaced, None) {
            return Err(denial);
        }
        match self.publication.claim_displaced(displaced.artifact) {
            GarbageClaim::Completed => Ok(None),
            GarbageClaim::Absent => Err(PhysicalRetirementDenial::Absent),
            GarbageClaim::Claimed(_) | GarbageClaim::AlreadyClaimed(_) => Ok(Some(displaced)),
        }
    }

    pub(in crate::physical_runtime) fn blocked_retirement(
        &self,
        displaced: &DisplacedArtifact,
    ) -> Option<PhysicalRetirementDenial> {
        let state = self.lock_publication_state();
        retirement_blocked(self, &state, displaced, None)
    }

    pub(in crate::physical_runtime) fn blocked_retirement_release(
        &self,
        displaced: &DisplacedArtifact,
        pending: &crate::physical_runtime::durability::PendingPublicationLease,
    ) -> Option<PhysicalRetirementDenial> {
        retirement_blocked(
            self,
            &self.lock_publication_state(),
            displaced,
            Some(pending),
        )
    }

    pub(in crate::physical_runtime) fn revert_displaced_claim(&self, artifact: RetiredArtifact) {
        self.publication.revert_displaced_claim(artifact);
    }

    /// Restores the precise claim carried by a durable retirement intent.
    /// The pending release still performs the protected-root check before any
    /// publication or removal effect.
    pub(in crate::physical_runtime) fn claim_recovered_retirement(
        &self,
        displaced: DisplacedArtifact,
    ) -> bool {
        self.publication.claim_recovered_displaced_exact(displaced)
    }

    pub(in crate::physical_runtime) fn complete_displaced(&self, artifact: RetiredArtifact) {
        self.publication.complete_displaced(artifact);
    }

    pub(in crate::physical_runtime) fn removal_permit(
        &self,
        artifact: RetiredArtifact,
    ) -> Option<crate::physical_runtime::durability::RetirementRemovalPermit> {
        self.publication.removal_permit(artifact)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn next_displaced(&self) -> Option<DisplacedArtifact> {
        self.publication.next_displaced()
    }
}

fn retirement_blocked(
    owner: &PhysicalCurrentRootOwner,
    state: &PhysicalCurrentRootState,
    displaced: &DisplacedArtifact,
    own: Option<&crate::physical_runtime::durability::PendingPublicationLease>,
) -> Option<PhysicalRetirementDenial> {
    let protected = match displaced.artifact {
        RetiredArtifact::Segment {
            segment,
            generation,
        } => {
            owner.read_protection.protects_root(displaced.source_root)
                || owner
                    .read_protection
                    .protects_inline_segment(segment, generation)
        }
        // Every root from the one that placed the extent generation through the
        // rewrite's source root reads it, so any reader at or below blocks.
        RetiredArtifact::Extent { .. } => owner
            .read_protection
            .protects_root_at_or_below(displaced.source_root),
        RetiredArtifact::Arena { .. } => owner
            .read_protection
            .protects_root_at_or_below(displaced.source_root.saturating_sub(1)),
    };
    if protected {
        return Some(PhysicalRetirementDenial::Protected);
    }
    if still_published(state, displaced) {
        return Some(PhysicalRetirementDenial::Retained);
    }
    let unresolved = own.map_or_else(
        || owner.publication.pending_len() > 0,
        |lease| owner.publication.pending_except(lease),
    );
    if unresolved {
        return Some(PhysicalRetirementDenial::Unresolved);
    }
    None
}

fn still_published(state: &PhysicalCurrentRootState, displaced: &DisplacedArtifact) -> bool {
    match displaced.artifact {
        RetiredArtifact::Segment {
            segment,
            generation,
        } => state
            .current_root
            .last_inline_segment()
            .is_some_and(|cell| {
                cell.segment_id().get() == segment && cell.generation().get() == generation
            }),
        // The rewrite that displaced the generation published its successor in
        // the root after source_root. Extent generations only move forward, so
        // once the current root is past the source root it no longer reads it.
        RetiredArtifact::Extent { .. } => state.current_root.generation() <= displaced.source_root,
        RetiredArtifact::Arena { generation, .. } => state.current_root.generation() < generation,
    }
}
