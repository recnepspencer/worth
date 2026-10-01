use std::num::NonZeroU64;

use worth_store::physical_runtime::{BlobReadLimits, CapabilityAvailability, PhysicalCapability};

use super::fixture::serving_from_initialization;

#[test]
fn constructed_serving_blob_facade_reports_only_installed_owners() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(parent.path());
    let status = serving.installed_capabilities();
    for installed in [
        PhysicalCapability::Media,
        PhysicalCapability::PageRecord,
        PhysicalCapability::WalCheckpoint,
        PhysicalCapability::Maintenance,
        PhysicalCapability::Blob,
        PhysicalCapability::Layout,
    ] {
        assert_eq!(
            status.availability(installed),
            CapabilityAvailability::Present
        );
    }
    assert_eq!(
        status.availability(PhysicalCapability::Recovery),
        CapabilityAvailability::Absent
    );
    let blobs = serving
        .blobs()
        .expect("constructed healthy Store has a blob owner");
    let object = blobs
        .issue_object_id(BlobReadLimits::new(NonZeroU64::new(1).unwrap()))
        .unwrap_or_else(|_| panic!("an empty selected root must permit bounded issuance"));
    assert_eq!(object.store(), serving.store_identity());
    assert_ne!(object.bytes(), [0; 16]);
    drop(blobs);
    serving.close();
}
