use worth_store::physical_runtime::{
    AdmittedPhysicalRecordResidencyPolicy, AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordinationCapacity, PhysicalRecoveryFreshnessAuthority,
};

fn cannot_coordinate(
    freshness: PhysicalRecoveryFreshnessAuthority,
    media: &AdmittedRecoveryFilesystemMedia,
    capacity: PhysicalRecoveryCoordinationCapacity,
    policy: AdmittedPhysicalRecordResidencyPolicy,
) {
    let _ = freshness.admit_coordination(media, capacity, policy, None);
}

fn main() {}
