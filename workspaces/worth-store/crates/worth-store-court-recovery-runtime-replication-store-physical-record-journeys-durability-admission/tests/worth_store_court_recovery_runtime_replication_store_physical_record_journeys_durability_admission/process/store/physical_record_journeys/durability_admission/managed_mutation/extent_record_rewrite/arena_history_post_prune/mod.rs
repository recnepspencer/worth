use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::segment_inventory::inspect_wal_inventory, worth_store_testkit_store::durability_admission::independent_wal_oracle::copy_intent::produced_copy_intents, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::arena_file, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare};

#[test]
fn deleted_arena_with_pruned_wal_is_not_claimed_from_unaddressed_root() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement_crash::kill_at_seam_with_policy(directory.path(), 2, true);
    let initial_wal = inspect_wal_inventory(&root).unwrap();
    let original_segments = initial_wal.segments().to_vec();
    assert_eq!(
        produced_copy_intents(&root).len(),
        1,
        "the killed copy originally retained one authenticated WAL intent"
    );
    assert_eq!(
        produced_retirement_payloads(&root)
            .iter()
            .filter(|record| record.kind == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Arena
                && record.action == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Intent)
            .count(),
        1,
        "the killed deletion originally retained one arena retirement intent"
    );
    let deleted = arena_file(&root, 1);
    assert!(
        !deleted.exists(),
        "the child was killed after real arena deletion"
    );
    let serving = crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement::reopen_with_small_copy_wal(&root);
    serving.retire_displaced_segment().unwrap();
    let (format, _, _) = configuration();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .arena_capacity(
            worth_store::physical_runtime::ExtentArenaCapacity::bytes(64 << 20).unwrap(),
        )
        .arena_evacuation(
            worth_store::physical_runtime::ArenaEvacuationThreshold::percent(99).unwrap(),
        )
        .admit(format)
        .unwrap();
    let rotating_payload = vec![73; 40_000];
    for ordinal in 3..=32 {
        completed(prepare(&serving, placement, [100 + ordinal; 32], &rotating_payload).execute());
        crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation::copy_checkpoint(&serving, ordinal);
        if produced_copy_intents(&root).is_empty() && produced_retirement_payloads(&root).is_empty()
        {
            break;
        }
    }
    serving.close();
    assert!(!deleted.exists());
    assert!(
        produced_copy_intents(&root).is_empty() && produced_retirement_payloads(&root).is_empty(),
        "the historical classification must not borrow reclaimed WAL provenance"
    );
    let remaining_wal = inspect_wal_inventory(&root).unwrap();
    assert!(
        original_segments
            .iter()
            .any(|segment| !remaining_wal.segments().contains(segment)),
        "a real checkpoint must reclaim at least one original WAL segment"
    );
    let selected = std::fs::read(root.join("families/records/root-current.selector")).unwrap();
    let previous = std::fs::read(root.join("families/records/root-previous.selector")).unwrap();
    let selected = u64::from_le_bytes(selected[65..73].try_into().unwrap());
    let previous = u64::from_le_bytes(previous[65..73].try_into().unwrap());
    assert!(previous > 3 && selected > previous);
    assert!(
        root.join("families/records/roots/root-0000000000000003.manifest")
            .exists(),
        "a predecessor root remains addressable after WAL reclamation"
    );
    let report = crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement_crash::observe_arena_report(
        &root,
        directory.path(),
        9,
        "post-prune-deleted-arena",
    );
    assert_eq!(report["completeness"], "complete", "{report:#?}");
    let accounting = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|artifact| {
            artifact["identity"]
                .as_str()
                .unwrap()
                .starts_with("arena-accounting:")
        })
        .collect::<Vec<_>>();
    // This proves no false claim after real WAL pruning, not the separate
    // Unknown path for a still-addressed predecessor gap.
    assert!(
        !accounting.iter().any(|artifact| artifact["path"]
            .as_str()
            .unwrap()
            .ends_with("arena-0000000000000001.data")),
        "root 3 remains on disk but is no longer addressed by either selector; this observation cannot claim its arena accounting: {accounting:#?}"
    );
    assert!(
        !accounting
            .iter()
            .any(|artifact| artifact["outcome"]["posture"] == "damaged"),
        "reclaimed history is not physical damage: {accounting:#?}"
    );
    for generation in [previous, selected] {
        assert!(
            accounting.iter().any(|artifact| artifact["identity"]
                .as_str()
                .unwrap()
                .starts_with(&format!("arena-accounting:{generation}:"))
                && artifact["path"]
                    .as_str()
                    .unwrap()
                    .ends_with("arena-0000000000000002.data")
                && artifact["outcome"]["posture"] == "intact"),
            "each addressed root accounts for its live destination: {accounting:#?}"
        );
    }
}
