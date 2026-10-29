use {worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest};
use {worth_store_physical_format::PhysicalTierClass, worth_store_physical_format::SelectedRecordContentClass};

use {worth_store_testkit_store::durable_publication::root::publish_single, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration, worth_store_testkit_store::physical_record_journeys::success};

#[test]
fn typed_ordinary_route_survives_fresh_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(2);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([198; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"typed ordinary route".as_slice()]).unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    let assert_route = |serving: &worth_store::physical_runtime::ServingPhysicalRuntime| {
        let mut scan = serving
            .records()
            .unwrap()
            .scan(worth_store::physical_runtime::RecordScanRequest::from_start())
            .unwrap();
        let mut scratch = [0_u8; 128];
        let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() else {
            panic!("the selected route must be discoverable");
        };
        assert_eq!(batch.records().len(), 1);
        assert_eq!(batch.records()[0].record_id(), record);
        assert_eq!(
            batch.records()[0].content_class(),
            worth_store_physical_format::SelectedRecordContentClass::Opaque
        );
        assert_eq!(batch.records()[0].tier_class(), worth_store_physical_format::PhysicalTierClass::Primary);
    };
    assert_route(&serving);
    serving.close();
    let reopened = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    }));
    assert_route(&reopened);
    reopened.close();
}
