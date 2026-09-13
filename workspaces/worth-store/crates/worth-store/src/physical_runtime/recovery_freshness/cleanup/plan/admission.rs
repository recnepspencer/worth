use std::sync::Arc;

use worth_store_physical_backend::{
    AdmittedRecoveryFilesystemMedia, PhysicalRecoveryMediaGeneration,
};
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, PhysicalCheckpointIdentity,
};
use worth_store_physical_integrity::VerifiedCheckpointStream;
use worth_store_wal::LogSequenceNumber;

use crate::physical_runtime::{
    ClosedPhysicalRecoveryCleanup, CompletedPhysicalRecoveryFreshReopen,
    PhysicalRecoveryCoordination,
};

use super::super::{StoreRecoveryCleanupFreshnessDenial, StoreRecoveryCleanupFreshnessFailure};
use super::{
    candidates, materialization, StoreRecoveryCleanupPlan, StoreRecoveryCleanupPlanAdmissionFailure,
};

pub(super) struct CommonBasis {
    pub(super) store: StableStoreIdentity,
    pub(super) media_generation: PhysicalRecoveryMediaGeneration,
    pub(super) session: [u8; 16],
    pub(super) published_generation: u64,
    pub(super) sealed_publication_basis: [u8; 32],
    pub(super) checkpoint: PhysicalCheckpointIdentity,
    pub(super) compaction_generation: u64,
    pub(super) compaction_digest: [u8; 32],
    pub(super) retained_boundary: LogSequenceNumber,
    pub(super) root_read: worth_store_physical_backend::CompletedScheduledRecoveryReopenRead,
}

pub(in crate::physical_runtime) fn admit(
    coordination: &PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    reopened: CompletedPhysicalRecoveryFreshReopen,
    checkpoint: Arc<VerifiedCheckpointStream>,
    descriptive_plan_identity: [u8; 32],
    wal: impl IntoIterator<Item = crate::physical_runtime::IntegrityAdmittedRecoveryWalSegment>,
) -> Result<StoreRecoveryCleanupPlan, StoreRecoveryCleanupPlanAdmissionFailure> {
    #[cfg(feature = "certification-test-authority")]
    if coordination.take_certification_cleanup_plan_admission_failure() {
        return Err(StoreRecoveryCleanupPlanAdmissionFailure {
            closed: ClosedPhysicalRecoveryCleanup::new(
                reopened,
                descriptive_plan_identity,
                None,
                0,
            ),
            failure: invalid(),
        });
    }
    let admitted = match candidates::admit(
        candidates::CandidateAdmissionContext {
            coordination,
            media,
            reopened: &reopened,
            checkpoint: &checkpoint,
            descriptive_plan_identity,
        },
        wal,
    ) {
        Ok(admitted) => admitted,
        Err(failure) => {
            return Err(admission_failure(
                reopened,
                descriptive_plan_identity,
                failure,
            ))
        }
    };
    Ok(materialization::materialize(
        media,
        materialization::PlanMaterializationInput {
            reopened,
            checkpoint,
            descriptive_plan_identity,
            admitted,
            capacity: coordination.cleanup_capacity(),
        },
    ))
}

pub(super) fn common_basis(
    coordination: &PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    reopened: &CompletedPhysicalRecoveryFreshReopen,
    checkpoint: &VerifiedCheckpointStream,
) -> Result<CommonBasis, StoreRecoveryCleanupFreshnessFailure> {
    let occurrence = reopened.fresh_reopen_occurrence();
    let root = reopened.root();
    let source = checkpoint.source();
    let checkpoint_root = source.root();
    let store = source.identity().store_identity();
    if store != media.store_identity()
        || occurrence.session() != coordination.session_identity()
        || checkpoint_root.generation() > root.generation()
        || checkpoint_root.tree_identity() != root.tree_identity()
    {
        return Err(invalid());
    }
    let compaction = checkpoint.compaction_cutover();
    Ok(CommonBasis {
        store,
        media_generation: media.media_generation(),
        session: occurrence.session(),
        published_generation: occurrence.generation(),
        sealed_publication_basis: occurrence.plan(),
        checkpoint: source.identity(),
        compaction_generation: compaction.product_generation(),
        compaction_digest: checkpoint.footer().binding_records_digest(),
        retained_boundary: LogSequenceNumber::new(source.wal().covered_end_lsn_exclusive()),
        root_read: occurrence.root().clone(),
    })
}

pub(super) fn invalid() -> StoreRecoveryCleanupFreshnessFailure {
    StoreRecoveryCleanupFreshnessFailure {
        denial: StoreRecoveryCleanupFreshnessDenial::InvalidCleanupEligibility,
        sample: None,
        read: None,
        binding: None,
        terminal_binding_evaluations: 0,
    }
}

fn admission_failure(
    reopened: CompletedPhysicalRecoveryFreshReopen,
    descriptive_plan_identity: [u8; 32],
    failure: StoreRecoveryCleanupFreshnessFailure,
) -> StoreRecoveryCleanupPlanAdmissionFailure {
    StoreRecoveryCleanupPlanAdmissionFailure {
        closed: ClosedPhysicalRecoveryCleanup::new(reopened, descriptive_plan_identity, None, 0),
        failure,
    }
}
