use {
    worth_store_testkit_store::durable_publication::root::publish_single,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn scan_payload_cap_defers_large_record_and_reaches_later_control() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(4);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    let payloads = [vec![31_u8; 120], vec![32_u8; 9_000], vec![33_u8; 120]];
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([186; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter(payloads.iter()).unwrap(),
    );
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(1).unwrap())
                .with_payload_limit(
                    worth_store::physical_runtime::RecordByteLimit::new(236).unwrap(),
                ),
        )
        .unwrap();
    let mut scratch = [0_u8; 236];
    for (index, expected) in payloads.iter().enumerate() {
        let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) =
            scan.read_next_into(&mut scratch).unwrap()
        else {
            panic!("the capped scan must advance across every selected identity");
        };
        assert_eq!(batch.records().len(), 1);
        assert_eq!(
            batch.records()[0].record_id(),
            published.settled_members()[0].record_id(index).unwrap()
        );
        assert_eq!(
            batch.records()[0].declared_payload_bytes(),
            expected.len() as u64
        );
        if index == 1 {
            assert!(batch.records()[0].payload_is_deferred());
            assert!(batch.payload(0).is_none());
        } else {
            assert_eq!(batch.payload(0), Some(expected.as_slice()));
        }
        assert_eq!(batch.is_complete(), index == 2);
    }
    serving.close();
}
