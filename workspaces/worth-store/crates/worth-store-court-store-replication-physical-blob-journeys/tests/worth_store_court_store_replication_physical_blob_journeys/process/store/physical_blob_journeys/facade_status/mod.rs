use worth_store_testkit_store::fixture::serving_from_initialization;

#[test]
fn constructed_serving_blob_facade_reports_only_installed_owners() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(parent.path());
    let status = serving.installed_capabilities();
    for installed in [
        worth_store::physical_runtime::PhysicalCapability::Media,
        worth_store::physical_runtime::PhysicalCapability::PageRecord,
        worth_store::physical_runtime::PhysicalCapability::WalCheckpoint,
        worth_store::physical_runtime::PhysicalCapability::Maintenance,
        worth_store::physical_runtime::PhysicalCapability::Blob,
        worth_store::physical_runtime::PhysicalCapability::Layout,
    ] {
        assert_eq!(
            status.availability(installed),
            worth_store::physical_runtime::CapabilityAvailability::Present
        );
    }
    assert_eq!(
        status.availability(worth_store::physical_runtime::PhysicalCapability::Recovery),
        worth_store::physical_runtime::CapabilityAvailability::Absent
    );
    let blobs = serving
        .blobs()
        .expect("constructed healthy Store has a blob owner");
    let object = blobs
        .issue_object_id(worth_store::physical_runtime::BlobReadLimits::new(
            std::num::NonZeroU64::new(1).unwrap(),
        ))
        .unwrap_or_else(|_| panic!("an empty selected root must permit bounded issuance"));
    assert_eq!(object.store(), serving.store_identity());
    assert_ne!(object.bytes(), [0; 16]);
    serving.close();
}
