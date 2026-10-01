//! Narrow root captures delegated to the Store publication owner.

use worth_store_physical_format::{DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest};

use super::RecordPublicationDirector;

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn capture_recovery_retained_root(
        &self,
    ) -> Result<
        Option<(
            DurablePhysicalRootManifest,
            crate::physical_runtime::stability::PhysicalRootReadLease,
        )>,
        crate::physical_runtime::PhysicalReadProtectionDenial,
    > {
        self.root_owner.capture_recovery_retained_root()
    }

    pub(in crate::physical_runtime) fn current_root(&self) -> DurablePhysicalRootManifest {
        self.root_owner.snapshot().0
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
        self.root_owner.capture_read_root()
    }

    pub(in crate::physical_runtime) fn capture_released_drop_source(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
    ) -> Result<
        (
            DurablePhysicalRootManifest,
            crate::physical_runtime::stability::PhysicalRootReadLease,
            DurableFreeSpaceManifestHeader,
        ),
        crate::physical_runtime::durability::ReleasedDropSourceCaptureDenial,
    > {
        self.root_owner.capture_released_drop_source(attempt)
    }

    pub(in crate::physical_runtime) fn capture_selected_released_drop_candidate(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
        completed: &crate::physical_runtime::CompletedPhysicalMutation,
    ) -> Result<
        (
            DurablePhysicalRootManifest,
            crate::physical_runtime::stability::PhysicalRootReadLease,
        ),
        crate::physical_runtime::durability::ReleasedDropSourceCaptureDenial,
    > {
        self.root_owner
            .capture_selected_released_drop_candidate(attempt, completed)
    }

    pub(in crate::physical_runtime) fn capture_blob_inspection(
        &self,
        session: crate::physical_runtime::BlobSessionId,
    ) -> Result<
        (
            DurablePhysicalRootManifest,
            crate::physical_runtime::stability::PhysicalRootReadLease,
            crate::physical_runtime::durability::PhysicalBlobSessionClaim,
        ),
        crate::physical_runtime::durability::PhysicalBlobSessionClaimDenial,
    > {
        self.root_owner.capture_blob_inspection(session)
    }
}
