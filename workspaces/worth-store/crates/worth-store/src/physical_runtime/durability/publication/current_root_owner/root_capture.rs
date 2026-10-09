use worth_store_physical_format::DurablePhysicalRootManifest;

#[cfg(feature = "certification-test-authority")]
use super::CertificationReadRootCaptureStage;
use super::PhysicalCurrentRootOwner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum ReleasedDropSourceCaptureDenial {
    ReclaimFenceMismatch,
    RootFreeSpaceMismatch,
    ReadProtection(crate::physical_runtime::PhysicalReadProtectionDenial),
}

impl PhysicalCurrentRootOwner {
    /// Captures the recovery-retained predecessor under the current-root lock.
    /// C.10 registers its lease before the root can be displaced or retired.
    pub(in crate::physical_runtime) fn capture_recovery_retained_root(
        &self,
    ) -> Result<
        Option<(
            DurablePhysicalRootManifest,
            crate::physical_runtime::stability::PhysicalRootReadLease,
        )>,
        crate::physical_runtime::PhysicalReadProtectionDenial,
    > {
        let state = self.lock_publication_state();
        let Some(previous) = state.previous_root.as_ref() else {
            return Ok(None);
        };
        let root = previous.manifest().clone();
        let lease = self.read_protection.capture(&root)?;
        Ok(Some((root, lease)))
    }

    /// A selected drop's own candidate root may be inspected while its reclaim
    /// fence remains held. Ordinary readers still cannot capture through it.
    pub(in crate::physical_runtime) fn capture_selected_released_drop_candidate(
        &self,
        attempt: &super::PhysicalReclaimAttempt,
        completed: &crate::physical_runtime::CompletedPhysicalMutation,
    ) -> Result<
        (
            DurablePhysicalRootManifest,
            crate::physical_runtime::stability::PhysicalRootReadLease,
        ),
        ReleasedDropSourceCaptureDenial,
    > {
        let state = self.lock_publication_state();
        let fence = self.lock_reclaim();
        let current = &state.current_root;
        if !fence.as_ref().is_some_and(|active| {
            active.matches_attempt(attempt.bytes())
                && active.is_selected_drop_mutation(completed.mutation_identity())
                && active.selected_root_cell() == current.root_cell()
        }) || completed.completed_breadth().current_root_generation() != current.generation()
        {
            return Err(ReleasedDropSourceCaptureDenial::ReclaimFenceMismatch);
        }
        let root = current.clone();
        let protection = self
            .read_protection
            .capture(&root)
            .map_err(ReleasedDropSourceCaptureDenial::ReadProtection)?;
        Ok((root, protection))
    }

    /// The immediate post-reservation, pre-drop source is captured under the
    /// same publication/reclaim lock pair as the selected reclaim fence.
    pub(in crate::physical_runtime) fn capture_released_drop_source(
        &self,
        attempt: &super::PhysicalReclaimAttempt,
    ) -> Result<
        (
            DurablePhysicalRootManifest,
            crate::physical_runtime::stability::PhysicalRootReadLease,
            worth_store_physical_format::DurableFreeSpaceManifestHeader,
        ),
        ReleasedDropSourceCaptureDenial,
    > {
        let state = self.lock_publication_state();
        let fence = self.lock_reclaim();
        if !fence.as_ref().is_some_and(|fence| {
            fence.matches_attempt(attempt.bytes())
                && fence.is_reservation_published_payload_drop()
                && fence.expected_root == state.current_root.root_cell()
        }) {
            return Err(ReleasedDropSourceCaptureDenial::ReclaimFenceMismatch);
        }
        if state.free_space.generation() != state.current_root.generation()
            || state.free_space.tree_identity() != state.current_root.tree_identity()
        {
            return Err(ReleasedDropSourceCaptureDenial::RootFreeSpaceMismatch);
        }
        let root = state.current_root.clone();
        let free = state.free_space.clone();
        let protection = self
            .read_protection
            .capture(&root)
            .map_err(ReleasedDropSourceCaptureDenial::ReadProtection)?;
        Ok((root, protection, free))
    }

    pub(in crate::physical_runtime) fn capture_read_root(
        &self,
    ) -> Result<
        (
            DurablePhysicalRootManifest,
            crate::physical_runtime::stability::PhysicalRootReadLease,
        ),
        crate::physical_runtime::PhysicalReadProtectionDenial,
    > {
        #[cfg(feature = "certification-test-authority")]
        self.pause_capture_at(CertificationReadRootCaptureStage::BeforeRootLock);
        let state = self.lock_publication_state();
        if self.lock_reclaim().is_some() {
            return Err(crate::physical_runtime::PhysicalReadProtectionDenial::ReclaimFenced);
        }
        let root = state.current_root.clone();
        #[cfg(feature = "certification-test-authority")]
        self.pause_capture_at(
            CertificationReadRootCaptureStage::AfterObservationBeforeRegistration,
        );
        let lease = self.read_protection.capture(&root)?;
        Ok((root, lease))
    }
}
