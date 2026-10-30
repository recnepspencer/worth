use worth_store::physical_runtime::{
    AdmittedRecoveryFilesystemMedia, ClosedPhysicalRecoveryCleanup,
    PhysicalRecoveryConstructionPort, PhysicalRecoveryCoordination,
};

fn reuse(
    first_coordination: PhysicalRecoveryCoordination,
    first_media: AdmittedRecoveryFilesystemMedia,
    second_coordination: PhysicalRecoveryCoordination,
    second_media: AdmittedRecoveryFilesystemMedia,
    closed: ClosedPhysicalRecoveryCleanup,
) {
    let _ = PhysicalRecoveryConstructionPort::construct(
        first_coordination,
        first_media,
        closed,
    );
    let _ = PhysicalRecoveryConstructionPort::construct(
        second_coordination,
        second_media,
        closed,
    );
}

fn main() {
    let _ = reuse;
}
