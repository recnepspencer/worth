use super::RecordPublicationDirector;
use worth_store_physical_format::{DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest};

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn charged_growth_bytes(&self) -> u64 {
        self.root_owner.charged_growth_bytes()
    }

    pub(in crate::physical_runtime) fn certification_owe_before_maintenance_barrier(&self) {
        self.wal.certification_owe_before_maintenance_barrier();
    }

    pub(in crate::physical_runtime) fn certification_stop_before_retirement_delete(&self) {
        self.stop_before_retirement_delete
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub(in crate::physical_runtime) fn certification_stop_after_retirement_delete(&self) {
        self.stop_after_retirement_delete
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub(in crate::physical_runtime) fn certification_public_segment_removal_rejected(
        &self,
    ) -> bool {
        let Some(displaced) = self.root_owner.next_displaced() else {
            return false;
        };
        self.root_work
            .certification_public_removal_rejected(displaced.artifact.files()[0])
            .unwrap_or(false)
    }

    pub(in crate::physical_runtime) fn pending_publication_count(&self) -> usize {
        self.root_owner.pending_publication_count()
    }

    pub(in crate::physical_runtime) fn limit_candidate_growth_bytes(
        &self,
        usable_growth_bytes: u64,
    ) {
        let headroom_bytes = 64 * 1024;
        let profile = crate::physical_runtime::durability::PhysicalRetentionProfile::new(
            usable_growth_bytes
                .checked_add(headroom_bytes)
                .expect("usable growth plus headroom fits u64"),
            4_096,
            headroom_bytes,
            8,
        )
        .expect("growth limits withhold nonzero progress headroom");
        self.root_owner.install_retention_profile(profile);
    }

    pub(in crate::physical_runtime) fn planning_snapshot(
        &self,
    ) -> (DurablePhysicalRootManifest, DurableFreeSpaceManifestHeader) {
        self.root_owner.snapshot()
    }

    pub(in crate::physical_runtime) fn fail_next_wal_member_before_effect(&self) {
        self.wal.fail_next_member_before_effect();
    }

    pub(in crate::physical_runtime) fn pause_mutation_at_for_certification(
        &self,
        checkpoint: crate::physical_runtime::durability::CertificationPhysicalMutationCheckpoint,
    ) -> crate::physical_runtime::durability::CertificationPhysicalMutationPauseGate {
        self.mutations.pause_at(checkpoint)
    }
}
