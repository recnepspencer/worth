use worth_store::physical_runtime::{
    ExternalPhysicalRecordLocator, RecordAppendBatch, RecordByteLimit, RecordReadLimits,
    ServingPhysicalRuntime,
};
use worth_store_physical_format::certification_crc32c_invocations;

use super::{
    configuration, durable_publication, read_record, serving_from_initialization, serving_from_open,
};

#[test]
fn ordinary_resident_lookup_hashes_cold_and_evicted_frames_but_not_warm_hits() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(&root);
    let payload = b"resident checksum mechanism evidence";
    let published = durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("resident-checksum-cost", 1),
        RecordAppendBatch::try_from_iter([payload.as_slice()]).unwrap(),
    );
    let locator = ExternalPhysicalRecordLocator::new(
        serving.store_identity(),
        published.settled_members()[0].record_id(0).unwrap(),
    );
    serving.close();
    let reopened = serving_from_open(&root);
    reopened
        .certification_physical_residency()
        .drain_unpinned_clean_frames();

    let cold_hashes = read_and_count_checksums(&reopened, locator, payload);
    assert!(
        cold_hashes > 0,
        "cold ordinary lookup must validate actual bytes"
    );
    for _ in 0..3 {
        assert_eq!(
            read_and_count_checksums(&reopened, locator, payload),
            0,
            "unchanged routing, segment membership, and page hits must not rehash"
        );
    }

    let drained = reopened
        .certification_physical_residency()
        .drain_unpinned_clean_frames();
    assert!(
        drained > 0,
        "the reload check requires actual resident frame removal"
    );
    assert_eq!(
        read_and_count_checksums(&reopened, locator, payload),
        cold_hashes,
        "reloaded generations must revalidate the same physical granules"
    );
    assert_eq!(read_and_count_checksums(&reopened, locator, payload), 0);
    reopened.close();
}

fn read_and_count_checksums(
    serving: &ServingPhysicalRuntime,
    locator: ExternalPhysicalRecordLocator,
    expected: &[u8],
) -> u64 {
    let before = certification_crc32c_invocations();
    let session = serving
        .records()
        .expect("read protection admission")
        .open_external(
            locator,
            RecordReadLimits::new(RecordByteLimit::new(1_024).unwrap()),
        )
        .unwrap();
    assert_eq!(read_record(session, expected.len()).0, expected);
    certification_crc32c_invocations() - before
}
