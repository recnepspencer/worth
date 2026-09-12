use std::io::Write;
use std::path::Path;

use worth_store::physical_runtime::{
    ExternalPhysicalRecordLocator, PhysicalRecordOpen, RecordAppendBatch, RecordByteLimit,
    RecordReadLimits,
};

use super::child_process::{hex, unhex};
use super::{configuration, durable_publication, serving_from_initialization, serving_from_open};

pub(super) fn allocation_writer(root: &Path, logical_bytes: &str) {
    let logical_bytes = logical_bytes.parse().unwrap();
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(root);
    let batch = RecordAppendBatch::builder()
        .push_source(super::stream_fixture::PatternSource::exact(logical_bytes))
        .build()
        .unwrap();
    let published = durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("cross-process-extent-allocation", 1),
        batch,
    );
    let published_member = &published.settled_members()[0];
    println!(
        "C5_ALLOC {} {} {}",
        serving
            .residency_observation()
            .counters()
            .peak_operation_bytes(),
        published_member.observation().peak_scratch_bytes(),
        hex(&ExternalPhysicalRecordLocator::new(
            serving.store_identity(),
            published_member.record_id(0).unwrap(),
        )
        .encode()),
    );
    std::io::stdout().flush().unwrap();
    serving.close();
}

pub(super) fn allocation_reader(root: &Path, encoded_locator: &str) {
    let serving = serving_from_open(root);
    let locator = ExternalPhysicalRecordLocator::decode(unhex(encoded_locator)).unwrap();
    let observation = {
        let mut record = serving
            .records()
            .expect("read protection admission")
            .open_external(
                locator,
                RecordReadLimits::new(RecordByteLimit::new(u32::MAX).unwrap()),
            )
            .unwrap();
        assert_eq!(record.read_next(&mut [0_u8; 1]).unwrap(), 1);
        record.observation()
    };
    println!(
        "C5_READ_ALLOC {} {}",
        serving
            .residency_observation()
            .counters()
            .peak_operation_bytes(),
        observation.peak_scratch_bytes(),
    );
    std::io::stdout().flush().unwrap();
    serving.close();
}

pub(super) fn extent_reader(root: &Path, encoded_locator: &str) {
    let serving = serving_from_open(root);
    let locator = ExternalPhysicalRecordLocator::decode(unhex(encoded_locator)).unwrap();
    let mut record = serving
        .records()
        .expect("read protection admission")
        .open_external(
            locator,
            RecordReadLimits::new(RecordByteLimit::new(u32::MAX).unwrap()),
        )
        .unwrap();
    let widths = [
        1_usize, 2, 3, 7, 31, 127, 255, 511, 1024, 2047, 4093, 8191, 16_384, 97, 5, 333, 65_535,
    ];
    let mut total = 0_u64;
    let mut digest = 0_u64;
    let mut turn = 0;
    loop {
        let mut buffer = vec![0_u8; widths[turn % widths.len()]];
        let count = record.read_next(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        for byte in &buffer[..count] {
            digest = digest.rotate_left(5) ^ u64::from(*byte);
        }
        total += count as u64;
        turn += 1;
    }
    println!("C5_EXTENT {total} {digest}");
    let observation = record.observation();
    println!(
        "C5_EXTENT_OBS {} {} {} {} {} {} {}",
        observation.bytes_requested(),
        observation.bytes_completed(),
        observation.transfer_count(),
        observation.peak_transfer_width(),
        observation.copied_bytes(),
        observation.generation_checks(),
        observation.generation_rejections(),
    );
    std::io::stdout().flush().unwrap();
    serving.close();
}

pub(super) fn scale_allocation_reader(root: &Path, encoded_locator: &str) {
    let format = super::scale_support::format();
    let access = super::scale_support::access(format, 7);
    let serving = super::success(open_record_store!(super::media(root), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }));
    let locator = ExternalPhysicalRecordLocator::decode(unhex(encoded_locator)).unwrap();
    let record = serving
        .records()
        .expect("read protection admission")
        .readmit_locator(locator)
        .into_result()
        .unwrap();
    serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(100).unwrap()),
        )
        .unwrap()
        .observation();
    let point = serving
        .residency_observation()
        .counters()
        .peak_operation_bytes();
    super::scale_support::complete_scan(&serving, 7, 16_384);
    let scan = serving
        .residency_observation()
        .counters()
        .peak_operation_bytes();
    super::scenario_evidence::emit_process("scale-allocation-probe", &serving);
    println!("C5_SCALE_ALLOC {point} {scan}");
    std::io::stdout().flush().unwrap();
    serving.close();
}
