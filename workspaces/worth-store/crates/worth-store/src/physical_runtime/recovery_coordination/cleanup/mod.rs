mod admission;
mod close;
mod freshness;
mod removal;

pub use freshness::{
    CompletedPhysicalRecoveryCleanupFreshnessRead, PhysicalRecoveryCleanupFreshnessReadDenial,
    PhysicalRecoveryCleanupFreshnessReadDenialKind, PhysicalRecoveryCleanupFreshnessReadOutcome,
    PhysicalRecoveryCleanupFreshnessReadProgress,
};
pub(in crate::physical_runtime) use removal::PhysicalRecoveryCleanupRemovalCommand;
pub use removal::{
    CompletedPhysicalRecoveryCleanupRemoval, PhysicalRecoveryCleanupRemovalDenial,
    PhysicalRecoveryCleanupRemovalDenialKind, PhysicalRecoveryCleanupRemovalIndeterminate,
    PhysicalRecoveryCleanupRemovalOutcome,
};

pub use admission::{
    PhysicalRecoveryCleanupAdmissionDenial, PhysicalRecoveryCleanupAdmissionDenialKind,
};
pub use close::ClosedPhysicalRecoveryCleanup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryCleanupCommandStage {
    FreshnessRead,
    Removal,
}

use super::PhysicalRecoveryCoordination;
use crate::physical_runtime::{
    CompletedPhysicalRecoveryFreshReopen, StoreRecoveryCleanupAttempt, StoreRecoveryCleanupPlan,
    StoreRecoveryCleanupPlanAdmissionFailure,
};

impl PhysicalRecoveryCoordination {
    pub fn admit_cleanup_plan(
        &self,
        media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
        reopened: CompletedPhysicalRecoveryFreshReopen,
        checkpoint: std::sync::Arc<worth_store_physical_integrity::VerifiedCheckpointStream>,
        descriptive_plan_identity: [u8; 32],
        wal: impl IntoIterator<Item = crate::physical_runtime::IntegrityAdmittedRecoveryWalSegment>,
    ) -> Result<StoreRecoveryCleanupPlan, StoreRecoveryCleanupPlanAdmissionFailure> {
        crate::physical_runtime::recovery_freshness::admit_cleanup_plan(
            self,
            media,
            reopened,
            checkpoint,
            descriptive_plan_identity,
            wal,
        )
    }

    /// Consumes a fresh reopen when the runtime elects to defer every optional
    /// cleanup effect. The resulting close is the only input accepted by
    /// recovered-runtime construction.
    pub fn defer_cleanup(
        &self,
        reopened: CompletedPhysicalRecoveryFreshReopen,
        descriptive_plan_identity: [u8; 32],
        live_media_handle_delta: u64,
    ) -> ClosedPhysicalRecoveryCleanup {
        ClosedPhysicalRecoveryCleanup::new(
            reopened,
            descriptive_plan_identity,
            None,
            live_media_handle_delta,
        )
    }

    pub(in crate::physical_runtime) fn read_cleanup_current_selector(
        &self,
        media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
        format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    ) -> PhysicalRecoveryCleanupFreshnessReadOutcome {
        freshness::read(self, media, format)
    }

    pub fn execute_cleanup_candidate(
        &self,
        media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
        plan: &mut StoreRecoveryCleanupPlan,
        artifact: worth_store_wal::WalSegmentArtifactIdentity,
    ) -> StoreRecoveryCleanupAttempt {
        let admission = match crate::physical_runtime::recovery_freshness::cleanup::sample(
            self.freshness(),
            self,
            media,
            plan,
            artifact,
        ) {
            Ok(admission) => admission,
            Err(failure) => return StoreRecoveryCleanupAttempt::FreshnessDenied(failure),
        };
        let (freshness, command) = admission.into_parts();
        match command {
            Some(command) => StoreRecoveryCleanupAttempt::Removal {
                freshness,
                outcome: removal::execute(self, media, command),
            },
            None => StoreRecoveryCleanupAttempt::PublishedGenerationChanged(freshness),
        }
    }
}
