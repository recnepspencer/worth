mod phase_three_support;

use phase_three_support::*;
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimits,
    PhysicalRecoveryPlatformAuthority, PhysicalRecoverySourceDenial,
    PhysicalRecoveryWalIntegrityObservationOutcome,
};

#[test]
fn bounded_genesis_discovery_selects_only_the_fixed_current_root() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    let discovered = admitted_recovery(&root).discover().unwrap();
    assert_eq!(discovered.counters().selector_slots, 2);
    assert_eq!(
        discovered.counters().current_selector_integrity_admissions,
        1
    );
    assert_eq!(discovered.counters().current_selector_interpretations, 1);
    assert_eq!(discovered.counters().current_root_integrity_admissions, 1);
    assert_eq!(
        discovered.counters().current_root_candidate_interpretations,
        1
    );
    assert_eq!(
        discovered.counters().previous_selector_integrity_admissions,
        0
    );
    assert_eq!(discovered.counters().previous_selector_interpretations, 0);
    assert_eq!(discovered.counters().root_candidates, 1);
    assert_eq!(discovered.counters().checkpoint_candidates, 0);
    assert_eq!(discovered.counters().wal_segments, 0);
    let selected = discovered.select().unwrap();
    assert_eq!(selected.store_identity(), store);
    assert_eq!(selected.root_generation(), 1);
    assert_eq!(
        selected.root_role(),
        worth_store_recovery_physics::SelectedPhysicalRootRole::Current
    );
    assert_eq!(selected.checkpoint_identity(), None);
    assert_eq!(selected.wal_segment_count(), 0);
    assert_eq!(selected.residue_count(), 0);
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Refused(refusal) =
        selected.cancel_before_reconstruction()
    else {
        panic!("pre-reconstruction cancellation must remain refused")
    };
    assert_eq!(refusal.recovery_effects(), 0);
}

#[test]
fn bounded_discovery_joins_root_bound_checkpoint_cutover_and_contiguous_wal() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    let checkpoint = publish_synthetic_checkpoint(&root, store);
    publish_synthetic_wal_tail(&root);
    let discovered = admitted_recovery(&root).discover().unwrap();
    assert_eq!(discovered.counters().checkpoint_candidates, 1);
    assert_eq!(discovered.counters().checkpoint_integrity_attempts, 5);
    assert_eq!(discovered.counters().checkpoint_integrity_admissions, 5);
    assert_eq!(discovered.counters().checkpoint_integrity_rejections, 0);
    assert_eq!(discovered.counters().checkpoint_owner_projections, 3);
    assert_eq!(discovered.counters().checkpoint_owner_decoder_entries, 0);
    assert_eq!(discovered.counters().wal_entries, 1);
    assert_eq!(discovered.counters().wal_segments, 1);
    assert_eq!(discovered.counters().wal_frames, 1);
    assert_eq!(discovered.counters().wal_integrity_attempts, 1);
    assert_eq!(discovered.counters().wal_integrity_admissions, 1);
    assert_eq!(discovered.counters().wal_integrity_rejections, 0);
    assert_eq!(discovered.counters().wal_owner_projections, 1);
    assert_eq!(discovered.counters().wal_owner_decoder_entries, 0);
    let selected = discovered.select().unwrap();
    assert_eq!(selected.checkpoint_identity(), Some(checkpoint));
    assert_eq!(selected.compaction_generation(), Some(1));
    assert_eq!(selected.wal_segment_count(), 1);
    assert_eq!(selected.wal_frame_count(), 1);
    assert_eq!(selected.wal_integrity_observations().len(), 1);
    assert_eq!(
        selected.wal_integrity_observations()[0].outcome(),
        PhysicalRecoveryWalIntegrityObservationOutcome::Admitted
    );
    assert_eq!(selected.residue_count(), 0);
    let _ = selected.cancel_before_reconstruction();
}

#[test]
fn nonempty_root_carries_its_manifest_addressed_extent_fact() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_nonempty_genesis(&root, store);
    let discovered = admitted_recovery(&root).discover().unwrap();
    assert_eq!(discovered.counters().manifest_blocks, 1);
    let selected = discovered.select().unwrap();
    assert_eq!(selected.selected_page_fact_count(), 2);
    assert_eq!(selected.distinct_page_and_extent_count(), 2);
    assert_eq!(selected.discovery_counters().selected_page_facts, 2);
    let _ = selected.cancel_before_reconstruction();
}

#[test]
fn total_observation_and_distinct_fact_limits_refuse_without_effects() {
    for constrain in ["observation", "distinct"] {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("store");
        let store = initialize_store(&root);
        publish_synthetic_nonempty_genesis(&root, store);
        let mut declaration = limit_declaration(2, 8, 8 * 1024);
        if constrain == "observation" {
            declaration.observation_bytes = 1;
        } else {
            declaration.distinct_pages_and_extents = 1;
        }
        let blocked = expect_blocked(
            admitted_recovery_with_limits(
                &root,
                PhysicalRecoveryLimits::admit(declaration).unwrap(),
            )
            .discover()
            .and_then(|discovered| discovered.select())
            .err()
            .unwrap(),
        );
        assert_eq!(blocked.kind, PhysicalRecoveryBlockKind::DiscoveryLimit);
        assert_eq!(blocked.recovery_effects(), 0);
    }
}

#[test]
fn repeated_selection_is_deterministic_and_never_promotes_wal_residue() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    let wal = root.join("families").join("wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(
        wal.join("plausible-newest.wal"),
        b"not a framed WAL segment",
    )
    .unwrap();
    let first = admitted_recovery(&root)
        .discover()
        .unwrap()
        .select()
        .unwrap();
    let first_facts = (
        first.root_generation(),
        first.wal_segment_count(),
        first.residue_count(),
    );
    let first_trace = first.source_trace();
    let _ = first.cancel_before_reconstruction();
    let second = admitted_recovery(&root)
        .discover()
        .unwrap()
        .select()
        .unwrap();
    assert_eq!(
        first_facts,
        (
            second.root_generation(),
            second.wal_segment_count(),
            second.residue_count(),
        )
    );
    assert_eq!(first_facts, (1, 0, 1));
    assert_eq!(second.source_trace(), first_trace);
    assert_eq!(first_trace.residue_count(), 1);
    let _ = second.cancel_before_reconstruction();
}

#[test]
fn each_discovery_bound_refuses_before_recovery_effects() {
    for (selector_candidates, wal_segments, manifest_bytes, expected) in [
        (1, 8, 8 * 1024, PhysicalRecoveryBlockKind::DiscoveryLimit),
        (2, 8, 1, PhysicalRecoveryBlockKind::DiscoveryLimit),
        (2, 1, 8 * 1024, PhysicalRecoveryBlockKind::DiscoveryLimit),
    ] {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("store");
        let store = initialize_store(&root);
        publish_synthetic_genesis(&root, store);
        if wal_segments == 1 {
            let wal = root.join("families").join("wal");
            std::fs::create_dir_all(&wal).unwrap();
            std::fs::write(wal.join("residue-a"), b"a").unwrap();
            std::fs::write(wal.join("residue-b"), b"b").unwrap();
        }
        let admitted = admitted_recovery_with_limits(
            &root,
            limits_for(selector_candidates, wal_segments, manifest_bytes),
        );
        let blocked = expect_blocked(admitted.discover().err().expect("limit crossing blocked"));
        assert_eq!(blocked.kind, expected);
        assert_eq!(blocked.recovery_effects(), 0);
    }
}

#[test]
fn absent_and_rejected_checkpoints_have_distinct_terminal_evidence() {
    let absent_parent = tempfile::tempdir().unwrap();
    let absent_root = absent_parent.path().join("store");
    let absent_store = initialize_store(&absent_root);
    publish_synthetic_genesis(&absent_root, absent_store);
    let absent = admitted_recovery(&absent_root).discover().unwrap();
    assert_eq!(absent.counters().checkpoints_absent, 1);
    assert_eq!(absent.counters().checkpoints_rejected, 0);
    let _ = absent.select().unwrap().cancel_before_reconstruction();

    let rejected_parent = tempfile::tempdir().unwrap();
    let rejected_root = rejected_parent.path().join("store");
    let rejected_store = initialize_store(&rejected_root);
    publish_synthetic_genesis(&rejected_root, rejected_store);
    std::fs::write(
        rejected_root.join("families").join("checkpoint.current"),
        b"corrupt checkpoint",
    )
    .unwrap();
    let before = PhysicalRecoveryPlatformAuthority::process_counters();
    let discovered = admitted_recovery(&rejected_root).discover().unwrap();
    assert_eq!(discovered.counters().checkpoints_absent, 0);
    assert_eq!(discovered.counters().checkpoints_rejected, 1);
    assert_eq!(discovered.counters().checkpoint_integrity_attempts, 1);
    assert_eq!(discovered.counters().checkpoint_integrity_admissions, 0);
    assert_eq!(discovered.counters().checkpoint_integrity_rejections, 1);
    assert_eq!(discovered.counters().checkpoint_owner_decoder_entries, 0);
    let blocked = expect_blocked(
        discovered
            .select()
            .err()
            .expect("rejected checkpoint must block"),
    );
    assert_eq!(blocked.kind, PhysicalRecoveryBlockKind::Checkpoint);
    assert_eq!(blocked.store_identity(), rejected_store);
    assert_eq!(blocked.evidence().counters.checkpoints_rejected, 1);
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::CheckpointIntegrity(
                worth_store_recovery_runtime::PhysicalRecoveryCheckpointIntegrityDenial::Integrity(
                    _
                )
            )
        )));
    assert_eq!(
        blocked.evidence().artifact.as_deref(),
        Some("families/checkpoint.current")
    );
    assert_eq!(blocked.recovery_effects(), 0);
    let after = PhysicalRecoveryPlatformAuthority::process_counters();
    assert!(after.sessions_terminated_blocked >= before.sessions_terminated_blocked + 1);
}

#[test]
fn cumulative_wal_bytes_stop_before_the_crossing_artifact() {
    let crossing_parent = tempfile::tempdir().unwrap();
    let crossing_root = crossing_parent.path().join("store");
    let crossing_store = initialize_store(&crossing_root);
    publish_synthetic_genesis(&crossing_root, crossing_store);
    write_two_eight_byte_residue_files(&crossing_root);
    let mut declaration = limit_declaration(2, 8, 8 * 1024);
    declaration.wal_bytes = 12;
    let blocked = expect_blocked(
        admitted_recovery_with_limits(
            &crossing_root,
            PhysicalRecoveryLimits::admit(declaration).unwrap(),
        )
        .discover()
        .err()
        .expect("cumulative WAL crossing must block"),
    );
    let limit = blocked.evidence().limit.unwrap();
    assert_eq!(limit.dimension, PhysicalRecoveryLimitDimension::WalBytes);
    assert_eq!((limit.observed, limit.admitted), (16, 12));
    assert_eq!(blocked.evidence().counters.wal_bytes, 8);
    assert_eq!(blocked.recovery_effects(), 0);

    let exact_parent = tempfile::tempdir().unwrap();
    let exact_root = exact_parent.path().join("store");
    let exact_store = initialize_store(&exact_root);
    publish_synthetic_genesis(&exact_root, exact_store);
    write_two_eight_byte_residue_files(&exact_root);
    let mut declaration = limit_declaration(2, 8, 8 * 1024);
    declaration.wal_bytes = 16;
    let discovered = admitted_recovery_with_limits(
        &exact_root,
        PhysicalRecoveryLimits::admit(declaration).unwrap(),
    )
    .discover()
    .unwrap();
    assert_eq!(discovered.counters().wal_bytes, 16);
    assert_eq!(discovered.counters().noncanonical_wal_residue, 2);
    let _ = discovered.select().unwrap().cancel_before_reconstruction();
}

#[test]
fn aggregate_manifest_entries_stop_before_the_crossing_leaf_is_extended() {
    let crossing_parent = tempfile::tempdir().unwrap();
    let crossing_root = crossing_parent.path().join("store");
    let crossing_store = initialize_store(&crossing_root);
    publish_synthetic_branched_genesis(&crossing_root, crossing_store);
    let mut declaration = limit_declaration(2, 8, 8 * 1024);
    declaration.manifest_entries = 2;
    let blocked = expect_blocked(
        admitted_recovery_with_limits(
            &crossing_root,
            PhysicalRecoveryLimits::admit(declaration).unwrap(),
        )
        .discover()
        .err()
        .expect("aggregate manifest entry crossing must block"),
    );
    assert_eq!(blocked.kind, PhysicalRecoveryBlockKind::DiscoveryLimit);
    let limit = blocked.evidence().limit.unwrap();
    assert_eq!(
        limit.dimension,
        PhysicalRecoveryLimitDimension::ManifestEntries
    );
    assert_eq!((limit.observed, limit.admitted), (3, 2));
    assert_eq!(blocked.recovery_effects(), 0);

    let exact_parent = tempfile::tempdir().unwrap();
    let exact_root = exact_parent.path().join("store");
    let exact_store = initialize_store(&exact_root);
    publish_synthetic_branched_genesis(&exact_root, exact_store);
    let mut declaration = limit_declaration(2, 8, 8 * 1024);
    declaration.manifest_entries = 3;
    let selected = admitted_recovery_with_limits(
        &exact_root,
        PhysicalRecoveryLimits::admit(declaration).unwrap(),
    )
    .discover()
    .unwrap()
    .select()
    .unwrap();
    assert_eq!(selected.selected_page_fact_count(), 3);
    assert_eq!(selected.discovery_counters().manifest_entries, 3);
    let _ = selected.cancel_before_reconstruction();
}

fn write_two_eight_byte_residue_files(root: &std::path::Path) {
    let wal = root.join("families").join("wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join("residue-a"), b"12345678").unwrap();
    std::fs::write(wal.join("residue-b"), b"abcdefgh").unwrap();
}
