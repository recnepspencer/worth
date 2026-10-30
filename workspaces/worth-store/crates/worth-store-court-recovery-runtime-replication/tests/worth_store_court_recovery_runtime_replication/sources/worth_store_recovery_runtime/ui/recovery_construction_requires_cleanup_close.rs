use worth_store::physical_runtime::{
    AdmittedRecoveryFilesystemMedia, CompletedPhysicalRecoveryFreshReopen,
    PhysicalRecoveryConstructionPort, PhysicalRecoveryCoordination,
};

fn bypass_cleanup(
    coordination: PhysicalRecoveryCoordination,
    media: AdmittedRecoveryFilesystemMedia,
    reopened: CompletedPhysicalRecoveryFreshReopen,
) {
    let _ = PhysicalRecoveryConstructionPort::construct(coordination, media, reopened);
}

fn main() {
    let _ = bypass_cleanup;
}
