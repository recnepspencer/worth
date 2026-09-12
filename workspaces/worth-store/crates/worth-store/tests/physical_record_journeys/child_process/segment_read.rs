use std::{io::Write, path::Path};

use worth_store::physical_runtime::{
    ExternalPhysicalRecordLocator, RecordByteLimit, RecordReadLimits,
};

use super::unhex;
use super::{super::serving_from_open, LOCATOR_ENV};

pub(super) fn run(root: &Path) {
    let serving = serving_from_open(root);
    for requested in std::env::var(LOCATOR_ENV).unwrap().split(';') {
        let (index, encoded) = requested.split_once(':').unwrap();
        let locator = ExternalPhysicalRecordLocator::decode(unhex(encoded)).unwrap();
        let mut record = serving
            .records()
            .expect("read protection admission")
            .open_external(
                locator,
                RecordReadLimits::new(RecordByteLimit::new(4_000).unwrap()),
            )
            .expect("C5_PREDICATE:identity-placement-seam");
        let mut bytes = vec![0_u8; 3_000];
        let mut completed = 0;
        while completed < bytes.len() {
            let count = record.read_next(&mut bytes[completed..]).unwrap();
            assert!(count > 0);
            completed += count;
        }
        assert_eq!(record.read_next(&mut [0_u8; 1]).unwrap(), 0);
        assert!(
            bytes.iter().all(|byte| *byte == bytes[0]),
            "C5_PREDICATE:identity-placement-seam"
        );
        let observation = record.observation();
        println!(
            "C5_SEGMENT {index} {} {} {} {}",
            bytes[0],
            completed,
            observation.touched_segments(),
            observation.touched_pages(),
        );
    }
    std::io::stdout().flush().unwrap();
    serving.close();
}
