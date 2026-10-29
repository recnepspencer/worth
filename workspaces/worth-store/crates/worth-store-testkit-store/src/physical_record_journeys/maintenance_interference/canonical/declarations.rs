pub use {std::io::Write};
pub use {std::path::Path};

pub use {worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalWorkCapacity};

pub use {crate::physical_record_journeys::maintenance_interference::append, crate::physical_record_journeys::maintenance_interference::append_copies, crate::physical_record_journeys::maintenance_interference::checkpoint, crate::physical_record_journeys::maintenance_interference::initialize, crate::physical_record_journeys::maintenance_interference::limits, crate::physical_record_journeys::maintenance_interference::placement, crate::physical_record_journeys::maintenance_interference::segment_files, crate::physical_record_journeys::maintenance_interference::segment_ids, crate::physical_record_journeys::maintenance_interference::RESIDENT_BYTES};

pub fn canonical_child(root: &Path) {
    let expected = std::env::var(crate::child_process::LOCATOR_ENV)
        .expect("baseline generation")
        .parse::<u64>()
        .unwrap();
    let (profile, request, _) = crate::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture();
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(8, 256, 32_768, 1024 * 1024, 64 * 1024 * 1024)
        .unwrap()
        .with_dispatch_permits(4)
        .unwrap();
    let (serving, gate, activation) =
        crate::physical_record_journeys::maintenance_interference::sync::reopen_with_file_sync_pause(root, profile.with_capacity(capacity));
    let generation = serving
        .observer()
        .acquisition_snapshot()
        .unwrap()
        .root_generation();
    assert_eq!(generation, expected);
    crate::physical_record_journeys::maintenance_interference::interleave::run_cold_extent(&serving, gate, activation, root, request, 100_000);
    let peak = serving
        .certification_physical_residency()
        .counters()
        .peak_resident_bytes();
    println!("PHASE6_CANONICAL {generation} {peak}");
    std::io::stdout().flush().unwrap();
    serving.close();
}

pub fn extend_a_partial_tail(
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

pub fn rotations(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) -> u64 {
    serving
        .record_submission()
        .wal_observation()
        .map(|observation| observation.rotations())
        .unwrap_or(0)
}