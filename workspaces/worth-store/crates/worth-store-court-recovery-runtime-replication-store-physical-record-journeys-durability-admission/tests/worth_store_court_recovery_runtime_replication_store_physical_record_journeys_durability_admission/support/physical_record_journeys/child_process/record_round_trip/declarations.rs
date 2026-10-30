pub(crate) use {std::io::Write, std::path::Path};

pub(crate) use {
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};

pub(crate) use {
    worth_store_testkit_store::child_process::locator_codec::hex,
    worth_store_testkit_store::child_process::locator_codec::unhex,
    worth_store_testkit_store::child_process::LOCATOR_ENV,
    worth_store_testkit_store::durable_publication,
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
    worth_store_testkit_store::physical_record_journeys::serving_from_open,
};

pub(crate) fn writer(root: &Path) {
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(root);
    let published = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "cross-process-record-round-trip",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"alpha".as_slice()])
            .unwrap(),
    );
    let first = worth_store::physical_runtime::ExternalPhysicalRecordLocator::new(
        serving.store_identity(),
        published.settled_members()[0].record_id(0).unwrap(),
    );
    let successor = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "cross-process-record-round-trip",
            2,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"beta".as_slice()])
            .unwrap(),
    );
    let second = worth_store::physical_runtime::ExternalPhysicalRecordLocator::new(
        serving.store_identity(),
        successor.settled_members()[0].record_id(0).unwrap(),
    );
    println!("C5_LOCATOR {}", hex(&first.encode()));
    println!("C5_LOCATOR_2 {}", hex(&second.encode()));
    std::io::stdout().flush().unwrap();
    std::process::exit(0);
}

pub(crate) fn reader(root: &Path) {
    let serving = serving_from_open(root);
    let locators = std::env::var(LOCATOR_ENV).unwrap();
    for (index, encoded) in locators.split(',').enumerate() {
        let locator =
            worth_store::physical_runtime::ExternalPhysicalRecordLocator::decode(unhex(encoded))
                .unwrap();
        let mut record = serving
            .records()
            .expect("read protection admission")
            .open_external(
                locator,
                worth_store::physical_runtime::RecordReadLimits::new(
                    worth_store::physical_runtime::RecordByteLimit::new(1024).unwrap(),
                ),
            )
            .unwrap();
        let label = if index == 0 {
            "C5_PAYLOAD"
        } else {
            "C5_PAYLOAD_2"
        };
        let mut bytes = vec![0_u8; 5_usize.saturating_sub(index)];
        let mut completed = 0;
        while completed < bytes.len() {
            let count = record.read_next(&mut bytes[completed..]).unwrap();
            assert!(count > 0);
            completed += count;
        }
        assert_eq!(record.read_next(&mut [0_u8; 1]).unwrap(), 0);
        println!("{label} {}", hex(&bytes));
    }
    serving.close();
}
