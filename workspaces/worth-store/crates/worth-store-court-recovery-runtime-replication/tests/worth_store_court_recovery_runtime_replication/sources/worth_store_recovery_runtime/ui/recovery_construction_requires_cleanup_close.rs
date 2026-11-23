use worth_store::physical_runtime::{
    AdmittedRecoveryFilesystemMedia, CompletedPhysicalRecoveryFreshReopen,
    PhysicalRecoveryConstructionPort, PhysicalRecoveryCoordination,
    PreparedRecoveredRuntimeCoreBacking,
};

fn bypass_cleanup(
    backing: PreparedRecoveredRuntimeCoreBacking,
    coordination: PhysicalRecoveryCoordination,
    media: AdmittedRecoveryFilesystemMedia,
    reopened: CompletedPhysicalRecoveryFreshReopen,
) {
    let _ = PhysicalRecoveryConstructionPort::construct_with_backing(
        backing,
        coordination,
        media,
        reopened,
    );
}

fn main() {
    let _ = bypass_cleanup;
}
