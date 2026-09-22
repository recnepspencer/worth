use super::PhysicalCurrentRootOwner;

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
            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
            Some(crate::physical_runtime::durability::retention::DisplacedSegment {
                source_root,
                segment_id,
                generation,
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
        let leases = std::mem::take(
            &mut *self
                .rewrite_growth
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        for lease in leases {
            self.publication.seal_candidate_charge(lease.generation());
        }
        if let Some(displaced) = self
            .displaced
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            self.publication.retain_displaced(displaced);
        }
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

    pub(in crate::physical_runtime) fn root_is_protected(&self, generation: u64) -> bool {
        self.read_protection.protects_root(generation)
    }

    pub(in crate::physical_runtime) fn next_displaced(
        &self,
    ) -> Option<crate::physical_runtime::durability::retention::DisplacedSegment> {
        self.publication.next_displaced()
    }

    pub(in crate::physical_runtime) fn claim_displaced(
        &self,
        generation: u64,
    ) -> crate::physical_runtime::durability::retention::GarbageClaim {
        self.publication.claim_displaced(generation)
    }

    pub(in crate::physical_runtime) fn revert_displaced_claim(&self, generation: u64) {
        self.publication.revert_displaced_claim(generation);
    }

    pub(in crate::physical_runtime) fn complete_displaced(&self, generation: u64) {
        self.publication.complete_displaced(generation);
    }
}
