use worth_store::physical_runtime::{
    AdmittedPhysicalRecordResidencyPolicy, AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordinationCapacity, PhysicalRecoveryRegisteredSessionAuthority,
};

fn duplicate_coordination(
    session: PhysicalRecoveryRegisteredSessionAuthority,
    media: &mut AdmittedRecoveryFilesystemMedia,
    capacity: PhysicalRecoveryCoordinationCapacity,
    policy: AdmittedPhysicalRecordResidencyPolicy,
) {
    let _first = session.admit_coordination(media, capacity, policy, None);
    let _second = session.admit_coordination(media, capacity, policy, None);
}

fn main() {}
