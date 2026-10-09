use worth_store::physical_runtime::{
    PhysicalRecordInitialization, PhysicalRecordOpen, RecordAppendBatch, RecordScanOutcome,
    RecordScanRequest,
};
use worth_store_physical_format::{PhysicalTierClass, SelectedRecordContentClass};

use super::super::{
    durable_publication::publish_single, media, scenario_configuration::dense_configuration,
    success,
};

#[test]
fn typed_ordinary_route_survives_fresh_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(2);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([198; 32]),
        RecordAppendBatch::try_from_iter([b"typed ordinary route".as_slice()]).unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    let assert_route = |serving: &worth_store::physical_runtime::ServingPhysicalRuntime| {
        let mut scan = serving
            .records()
            .unwrap()
            .scan(RecordScanRequest::from_start())
            .unwrap();
        let mut scratch = [0_u8; 128];
        let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() else {
            panic!("the selected route must be discoverable");
        };
        assert_eq!(batch.records().len(), 1);
        assert_eq!(batch.records()[0].record_id(), record);
        assert_eq!(
            batch.records()[0].content_class(),
            SelectedRecordContentClass::Opaque
        );
        assert_eq!(batch.records()[0].tier_class(), PhysicalTierClass::Primary);
    };
    assert_route(&serving);
    serving.close();
    let reopened = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }));
    assert_route(&reopened);
    reopened.close();
}
