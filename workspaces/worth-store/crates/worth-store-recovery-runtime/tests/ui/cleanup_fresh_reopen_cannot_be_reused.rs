use worth_store::physical_runtime::{
    AdmittedRecoveryFilesystemMedia, CompletedPhysicalRecoveryFreshReopen,
    IntegrityAdmittedRecoveryWalSegment, PhysicalRecoveryCoordination,
};
use worth_store_physical_integrity::VerifiedCheckpointStream;
use std::sync::Arc;

fn reuse_fresh_reopen(
    coordination: &PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    reopened: CompletedPhysicalRecoveryFreshReopen,
    checkpoint: Arc<VerifiedCheckpointStream>,
) {
    let _first = coordination.admit_cleanup_plan(
        media,
        reopened,
        checkpoint.clone(),
        [0x11; 32],
        std::iter::empty::<IntegrityAdmittedRecoveryWalSegment>(),
    );
    let _second = coordination.admit_cleanup_plan(
        media,
        reopened,
        checkpoint,
        [0x22; 32],
        std::iter::empty::<IntegrityAdmittedRecoveryWalSegment>(),
    );
}

fn main() {
    let _ = reuse_fresh_reopen;
}
