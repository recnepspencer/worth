#[allow(unused_imports)]
use worth_store_testkit_store::physical_record_journeys::maintenance_interference::interleave::*;

#[test]
fn cold_extent_survives_four_publications_scrub_and_blocked_retirement() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (profile, request, _) = worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture();
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(
        8,
        256,
        32_768,
        1024 * 1024,
        64 * 1024 * 1024,
    )
    .unwrap()
    .with_dispatch_permits(4)
    .unwrap();
    let (serving, gate, activation) =
        worth_store_testkit_store::physical_record_journeys::maintenance_interference::sync::open_with_file_sync_pause(&root, profile.with_capacity(capacity));
    run_cold_extent(&serving, gate, activation, &root, request, 0);
    serving.close();
}
