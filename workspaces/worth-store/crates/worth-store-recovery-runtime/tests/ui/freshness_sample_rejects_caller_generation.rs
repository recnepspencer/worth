use worth_store::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrameView, PhysicalRecoveryFreshnessPort,
};

fn substitute_generation(
    media: &worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia,
    checkpoint: &worth_store_physical_integrity::VerifiedCheckpointFacts,
) {
    let _ = PhysicalRecoveryFreshnessPort::sample_binding(
        7_u64,
        media,
        checkpoint,
        IntegrityAdmittedRecoveryWalFrameView::from_frames(&[]),
        1,
        1,
        1,
    );
}

fn main() {}
