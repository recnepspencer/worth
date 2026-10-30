#[allow(unused_imports)]
use crate::physical_record_journeys::resident_checksum_cost::*;

use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
    worth_store_testkit_store::physical_record_journeys::serving_from_open,
};

#[test]
fn ordinary_resident_lookup_hashes_cold_and_evicted_frames_but_not_warm_hits() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(&root);
    let payload = b"resident checksum mechanism evidence";
    let published = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "resident-checksum-cost",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload.as_slice()])
            .unwrap(),
    );
    let locator = worth_store::physical_runtime::ExternalPhysicalRecordLocator::new(
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
