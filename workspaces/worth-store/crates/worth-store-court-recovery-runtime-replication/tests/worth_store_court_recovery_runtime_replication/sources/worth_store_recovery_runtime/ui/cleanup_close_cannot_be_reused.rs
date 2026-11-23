use worth_store::physical_runtime::{
    AdmittedRecoveryFilesystemMedia, ClosedPhysicalRecoveryCleanup,
    PhysicalRecoveryConstructionPort, PhysicalRecoveryCoordination,
    PreparedRecoveredRuntimeCoreBacking,
};

fn reuse(
    first_backing: PreparedRecoveredRuntimeCoreBacking,
    first_coordination: PhysicalRecoveryCoordination,
    first_media: AdmittedRecoveryFilesystemMedia,
    second_backing: PreparedRecoveredRuntimeCoreBacking,
    second_coordination: PhysicalRecoveryCoordination,
    second_media: AdmittedRecoveryFilesystemMedia,
    closed: ClosedPhysicalRecoveryCleanup,
) {
    let _ = PhysicalRecoveryConstructionPort::construct_with_backing(
        first_backing,
        first_coordination,
        first_media,
        closed,
    );
    let _ = PhysicalRecoveryConstructionPort::construct_with_backing(
        second_backing,
        second_coordination,
        second_media,
        closed,
    );
}

fn main() {
    let _ = reuse;
}
