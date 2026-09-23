use std::io::Write;
use std::path::Path;

use worth_store::physical_runtime::{PhysicalRecordFormatDeclaration, PhysicalWorkCapacity};

use super::{
    append, append_copies, checkpoint, initialize, limits, placement, segment_files, segment_ids,
    RESIDENT_BYTES,
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
        .join("families/records/extents")
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

    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let walk = super::super::manifest_fixture::decode_routing_tree(&root, generation, format, 4);
    let level = walk.routing_level().unwrap_or(0);
    assert!(level >= 1, "routing height {level} is not multilevel");

    let output = super::super::child_process::run_child(
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

pub(crate) fn canonical_child(root: &Path) {
    let expected = std::env::var(super::super::child_process::LOCATOR_ENV)
        .expect("baseline generation")
        .parse::<u64>()
        .unwrap();
    let (profile, request, _) = super::super::physical_work::work_fixture();
    let capacity = PhysicalWorkCapacity::new(8, 256, 32_768, 1024 * 1024, 64 * 1024 * 1024)
        .unwrap()
        .with_dispatch_permits(4)
        .unwrap();
    let (serving, gate, activation) =
        super::sync::reopen_with_file_sync_pause(root, profile.with_capacity(capacity));
    let generation = serving
        .observer()
        .acquisition_snapshot()
        .unwrap()
        .root_generation();
    assert_eq!(generation, expected);
    super::interleave::run_cold_extent(&serving, gate, activation, root, request, 100_000);
    let peak = serving
        .certification_physical_residency()
        .counters()
        .peak_resident_bytes();
    println!("PHASE6_CANONICAL {generation} {peak}");
    std::io::stdout().flush().unwrap();
    serving.close();
}

fn extend_a_partial_tail(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    root: &Path,
    ordinal: &mut u64,
) {
    let ids = segment_ids(root);
    let files = segment_files(root);
    let tail = append(serving, policy, *ordinal, b"phase6-tail");
    *ordinal += 1;
    let mut session = serving.records().unwrap().open(tail, limits()).unwrap();
    assert_eq!(
        session.next_chunk().unwrap().unwrap().bytes(),
        b"phase6-tail"
    );
    drop(session);
    if segment_ids(root) == ids {
        assert_ne!(
            segment_files(root),
            files,
            "the tail append did not extend a segment"
        );
        return;
    }
    let opened = segment_ids(root);
    let opened_files = segment_files(root);
    append(serving, policy, *ordinal, b"phase6-tail-more");
    *ordinal += 1;
    assert_eq!(
        segment_ids(root),
        opened,
        "the follow-up append must extend the partial tail"
    );
    assert_ne!(segment_files(root), opened_files);
}

fn rotations(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) -> u64 {
    serving
        .record_submission()
        .wal_observation()
        .map(|observation| observation.rotations())
        .unwrap_or(0)
}
