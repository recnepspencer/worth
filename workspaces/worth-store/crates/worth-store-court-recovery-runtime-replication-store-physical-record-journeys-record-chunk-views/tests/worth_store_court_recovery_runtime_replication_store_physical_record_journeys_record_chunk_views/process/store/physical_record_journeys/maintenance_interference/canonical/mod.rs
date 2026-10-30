use std::io::Write;
use std::path::Path;
#[allow(unused_imports)]
use worth_store_testkit_store::physical_record_journeys::maintenance_interference::canonical::*;

use {
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalWorkCapacity,
};

use {
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::append,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::append_copies,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::checkpoint,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::limits,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_files,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_ids,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::RESIDENT_BYTES,
};

#[test]
fn canonical_world_reopens_into_the_cold_extent_interleave() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = initialize(&root);
    let policy = placement();
    let mut payload = 0_u64;
    let mut ordinal = 1_u64;
    let mut checkpointed_rotations = 0_u64;
    let record = [0x11; 4096];
    while payload < 2 * 1024 * 1024 || segment_ids(&root).len() < 8 || rotations(&serving) < 2 {
        if append_copies(&serving, policy, ordinal, &record, 4).is_none() {
            checkpoint(&serving, ordinal);
            ordinal += 1;
            assert!(
                append_copies(&serving, policy, ordinal, &record, 4).is_some(),
                "batch {ordinal} stayed denied after reclaim"
            );
        }
        payload += 4 * record.len() as u64;
        let seen = rotations(&serving);
        if seen > checkpointed_rotations {
            checkpoint(&serving, ordinal + 20_000);
            checkpointed_rotations = seen;
        }
        ordinal += 1;
        assert!(ordinal < 400, "world did not reach the scale predicates");
    }
    append(&serving, policy, ordinal, &[0x5A; 20_000]);
    ordinal += 1;
    assert!(root
        .join("families/records/arenas")
        .read_dir()
        .unwrap()
        .next()
        .is_some());
    extend_a_partial_tail(&serving, policy, &root, &mut ordinal);
    checkpoint(&serving, ordinal);
    let generation = serving
        .observer()
        .acquisition_snapshot()
        .unwrap()
        .root_generation();
    let resident = serving
        .certification_physical_residency()
        .counters()
        .peak_resident_bytes();
    assert!(
        resident <= RESIDENT_BYTES,
        "resident frames peaked at {resident}"
    );
    assert!(payload >= 32 * RESIDENT_BYTES);
    assert!(segment_ids(&root).len() >= 8);
    assert!(rotations(&serving) >= 2);
    serving.close();

    let format = worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let walk =
        worth_store_testkit_store::physical_record_journeys::manifest_fixture::decode_routing_tree(
            &root, generation, format, 4,
        );
    let level = walk.routing_level().unwrap_or(0);
    assert!(level >= 1, "routing height {level} is not multilevel");

    let output =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "interference_canonical_child",
            &root,
            Some(&generation.to_string()),
        );
    let line = output
        .lines()
        .find(|line| line.starts_with("PHASE6_CANONICAL "))
        .expect("serving child must finish the interleave");
    let mut fields = line.split_whitespace();
    assert_eq!(fields.next(), Some("PHASE6_CANONICAL"));
    let child_generation: u64 = fields.next().unwrap().parse().unwrap();
    let peak: u64 = fields.next().unwrap().parse().unwrap();
    assert_eq!(child_generation, generation);
    assert!(
        peak <= RESIDENT_BYTES,
        "child resident frames peaked at {peak}"
    );
}
