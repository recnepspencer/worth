use worth_store::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrameView, PhysicalRecoveryFreshnessPort,
};

fn substitute_generation(
    media: &worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia,
    basis: worth_store::physical_runtime::StoreRecoverySamplingBasis<'_>,
) {
    let _ = PhysicalRecoveryFreshnessPort::sample_binding(
        7_u64,
        media,
        basis,
        IntegrityAdmittedRecoveryWalFrameView::from_frames(&[]),
        1,
        1,
        1,
    );
}

fn main() {}
