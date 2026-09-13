use worth_store::physical_runtime::{
    AdmittedRecoveryFilesystemMedia, PhysicalRecoveryCoordinationCapacity,
    PhysicalRecoveryRegisteredSessionAuthority,
};

fn duplicate_coordination(
    session: PhysicalRecoveryRegisteredSessionAuthority,
    media: &mut AdmittedRecoveryFilesystemMedia,
    capacity: PhysicalRecoveryCoordinationCapacity,
) {
    let _first = session.admit_coordination(media, capacity, None);
    let _second = session.admit_coordination(media, capacity, None);
}

fn main() {}
