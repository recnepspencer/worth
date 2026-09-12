use std::{io::Write, path::Path};

use worth_store::physical_runtime::{
    ExternalPhysicalRecordLocator, RecordAppendBatch, RecordByteLimit, RecordReadLimits,
};

use super::{
    super::{configuration, durable_publication, serving_from_initialization, serving_from_open},
    hex, unhex, LOCATOR_ENV,
};

pub(super) fn writer(root: &Path) {
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(root);
    let published = durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("cross-process-record-round-trip", 1),
        RecordAppendBatch::try_from_iter([b"alpha".as_slice()]).unwrap(),
    );
    let first = ExternalPhysicalRecordLocator::new(
        serving.store_identity(),
        published.settled_members()[0].record_id(0).unwrap(),
    );
    let successor = durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("cross-process-record-round-trip", 2),
        RecordAppendBatch::try_from_iter([b"beta".as_slice()]).unwrap(),
    );
    let second = ExternalPhysicalRecordLocator::new(
        serving.store_identity(),
        successor.settled_members()[0].record_id(0).unwrap(),
    );
    println!("C5_LOCATOR {}", hex(&first.encode()));
    println!("C5_LOCATOR_2 {}", hex(&second.encode()));
    std::io::stdout().flush().unwrap();
    std::process::exit(0);
}

pub(super) fn reader(root: &Path) {
    let serving = serving_from_open(root);
    let locators = std::env::var(LOCATOR_ENV).unwrap();
    for (index, encoded) in locators.split(',').enumerate() {
        let locator = ExternalPhysicalRecordLocator::decode(unhex(encoded)).unwrap();
        let mut record = serving
            .records()
            .expect("read protection admission")
            .open_external(
                locator,
                RecordReadLimits::new(RecordByteLimit::new(1024).unwrap()),
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
