pub use {std::io::Write, std::path::Path};

pub use {
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};

pub use {
    crate::child_process::locator_codec::hex, crate::child_process::locator_codec::unhex,
    crate::child_process::LOCATOR_ENV, crate::durable_publication,
    crate::physical_record_journeys::configuration,
    crate::physical_record_journeys::serving_from_initialization,
    crate::physical_record_journeys::serving_from_open,
};

pub fn writer(root: &Path) {
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(root);
    let published = crate::durable_publication::root::publish_single(
        &serving,
        placement,
        crate::durable_publication::material::certification_material(
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
    let successor = crate::durable_publication::root::publish_single(
        &serving,
        placement,
        crate::durable_publication::material::certification_material(
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

pub fn reader(root: &Path) {
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
