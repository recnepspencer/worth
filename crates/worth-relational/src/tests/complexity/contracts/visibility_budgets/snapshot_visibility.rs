use crate::facade::history::BranchId;
use crate::tests::support::*;

#[test]
fn complexity_budget_snapshot_visibility_state_avoids_record_materialization() {
    let runtime = runtime_with_test_schema();
    let _ = create_entity(&runtime, "first");
    let _ = create_entity(&runtime, "second");

    runtime.performance_access().reset_counters();
    let _snapshot = runtime.visibility_authority().snapshot();
    let counters = runtime.performance_access().counters();

    assert_eq!(
        counters.visible_authoritative_entity_records_materialized,
        0
    );
    assert_eq!(
        counters.visible_authoritative_relation_records_materialized,
        0
    );
}

#[test]
fn complexity_budget_snapshot_release_uses_only_the_carried_root_obligation() {
    let runtime = runtime_with_test_schema();
    for index in 0..6 {
        let _ = create_entity(&runtime, &format!("e{index}"));
    }
    let snapshot = runtime.visibility_authority().snapshot();
    let target = create_entity(&runtime, "target");

    runtime.performance_access().reset_counters();
    let _ = update_entity(&runtime, target, "updated");
    let after_commit = runtime.performance_access().counters();
    assert_eq!(after_commit.snapshot_pin_full_rebuilds, 0);

    runtime.performance_access().reset_counters();
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&snapshot)
        .is_ok());
    let after_release = runtime.performance_access().counters();
    assert_eq!(after_release.snapshot_pin_full_rebuilds, 0);
    assert_eq!(after_release.snapshot_pin_adjustments, 0);
    assert_eq!(after_release.visibility_entity_slot_scans, 0);
    assert_eq!(after_release.visibility_relation_slot_scans, 0);
}

#[test]
fn complexity_budget_active_snapshots_share_one_constant_time_root_lease_per_version() {
    let runtime = runtime_with_test_schema();
    for index in 0..6 {
        let _ = create_entity(&runtime, &format!("e{index}"));
    }

    runtime.performance_access().reset_counters();
    let first = runtime.visibility_authority().snapshot();
    let first_open = runtime.performance_access().counters();
    assert_eq!(first_open.snapshot_pin_adjustments, 0);
    assert_eq!(first_open.visibility_entity_slot_scans, 0);
    assert_eq!(first_open.visibility_relation_slot_scans, 0);
    assert_eq!(first_open.visibility_cache_snapshot_promotions, 1);

    runtime.performance_access().reset_counters();
    let second = runtime.visibility_authority().snapshot();
    let second_open = runtime.performance_access().counters();
    assert_eq!(second_open.snapshot_pin_adjustments, 0);
    assert_eq!(second_open.visibility_cache_snapshot_promotions, 0);

    runtime.performance_access().reset_counters();
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&first)
        .is_ok());
    let first_release = runtime.performance_access().counters();
    assert_eq!(first_release.snapshot_pin_adjustments, 0);

    runtime.performance_access().reset_counters();
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&second)
        .is_ok());
    let second_release = runtime.performance_access().counters();
    assert_eq!(second_release.snapshot_pin_adjustments, 0);
    assert_eq!(second_release.visibility_entity_slot_scans, 0);
    assert_eq!(second_release.visibility_relation_slot_scans, 0);
}

#[test]
fn active_snapshot_registry_admission_and_release_work_is_population_independent() {
    let mut observed_work = Vec::new();
    for population in [1, 64, 4_096] {
        let runtime = snapshot_registry_scale_runtime();
        let _ = create_entity(&runtime, "scale-anchor");
        let seed = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
        let template = runtime
            .visibility
            .handles
            .active_binding(seed.snapshot_id())
            .expect("one real active snapshot supplies the registry binding shape")
            .clone();
        runtime
            .visibility_authority()
            .release_snapshot(&seed)
            .expect("the template active snapshot releases");
        for _ in 0..population {
            let snapshot_id = runtime
                .visibility
                .handles
                .next_snapshot_id()
                .expect("scale fixture snapshot identities remain bounded");
            runtime
                .visibility
                .handles
                .insert_active(snapshot_id, template.clone());
        }
        let before = runtime.visibility.snapshot_handle_registry_cost_counters();
        let target = runtime
            .visibility
            .handles
            .next_snapshot_id()
            .expect("target snapshot identity remains bounded");
        runtime
            .visibility
            .handles
            .insert_active(target, template.clone());
        let admitted = runtime.visibility.snapshot_handle_registry_cost_counters();
        runtime
            .visibility
            .handles
            .remove_active(target)
            .expect("one exact active registry handle releases");
        let released = runtime.visibility.snapshot_handle_registry_cost_counters();

        assert_eq!(before.active_entries, population as u64);
        assert_eq!(admitted.active_entries, population as u64 + 1);
        assert_eq!(released.active_entries, population as u64);
        observed_work.push((
            admitted.active_key_lookups - before.active_key_lookups,
            admitted.active_mutations - before.active_mutations,
            released.active_key_lookups - admitted.active_key_lookups,
            released.active_mutations - admitted.active_mutations,
        ));
    }
    assert!(observed_work[0].0 > 0);
    assert!(observed_work[0].2 > 0);
    assert!(observed_work.iter().all(|work| *work == observed_work[0]));
}

#[test]
fn published_snapshot_registry_admission_and_release_work_is_population_independent() {
    let mut observed_work = Vec::new();
    for population in [1, 64, 4_096] {
        let runtime = snapshot_registry_scale_runtime();
        let seed = create_entity_outcome(&runtime, "scale-template");
        let template = runtime
            .visibility
            .handles
            .published_binding(seed.snapshot.snapshot_id())
            .expect("one real publication supplies the registry binding shape");
        release_test_commit_snapshot(&runtime, &seed);
        for index in 0..population {
            let snapshot_id = runtime
                .visibility
                .handles
                .next_snapshot_id()
                .expect("scale fixture snapshot identities remain bounded");
            runtime.visibility.handles.insert_published(
                snapshot_id,
                template.for_registry_scale_test(crate::facade::identity::VersionId(
                    1_000_000 + index as u64,
                )),
            );
        }
        let before = runtime.visibility.snapshot_handle_registry_cost_counters();
        let target = runtime
            .visibility
            .handles
            .next_snapshot_id()
            .expect("target snapshot identity remains bounded");
        runtime.visibility.handles.insert_published(
            target,
            template.for_registry_scale_test(crate::facade::identity::VersionId(
                2_000_000 + population as u64,
            )),
        );
        let admitted = runtime.visibility.snapshot_handle_registry_cost_counters();
        runtime
            .visibility
            .handles
            .remove_published(target)
            .expect("one exact published registry handle releases");
        let released = runtime.visibility.snapshot_handle_registry_cost_counters();

        assert_eq!(before.published_entries, population as u64);
        assert_eq!(admitted.published_entries, population as u64 + 1);
        assert_eq!(released.published_entries, population as u64);
        observed_work.push((
            admitted.published_key_lookups - before.published_key_lookups,
            admitted.published_mutations - before.published_mutations,
            released.published_key_lookups - admitted.published_key_lookups,
            released.published_mutations - admitted.published_mutations,
        ));
    }
    assert!(observed_work[0].0 > 0);
    assert!(observed_work[0].2 > 0);
    assert!(observed_work.iter().all(|work| *work == observed_work[0]));
}

fn snapshot_registry_scale_runtime() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .publication(PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4_096,
            max_published_snapshot_handles: 8_192,
            max_active_snapshot_handles: 8_192,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 8,
            candidate_max_lifetime_millis: 30_000,
            max_prepared_root_bytes: 268_435_456,
        })
        .build()
}

#[test]
fn complexity_budget_branch_creation_reuses_cached_visibility_state() {
    let runtime = runtime_with_test_schema();
    let left = create_entity(&runtime, "left");
    let right = create_entity(&runtime, "right");
    let _ = create_relation(&runtime, left, right, "r0");

    runtime.performance_access().reset_counters();
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.visibility_entity_slot_scans, 0);
    assert_eq!(counters.visibility_relation_slot_scans, 0);
}

#[test]
fn complexity_contract_visibility_scans_are_explicitly_measured() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let relation_outcome = create_relation_outcome(&runtime, source, target, "r0");
    let snapshot = runtime.visibility_authority().snapshot();
    let historical_version = relation_outcome.version_id;
    let current_version = create_entity_outcome(&runtime, "later").version_id;

    runtime.performance_access().reset_counters();
    let _ = runtime.read_truth().read_snapshot(&snapshot).unwrap();
    let snapshot_counters = runtime.performance_access().counters();

    assert_eq!(snapshot_counters.visibility_entity_slot_scans, 0);
    assert_eq!(snapshot_counters.visibility_relation_slot_scans, 0);
    assert!(snapshot_counters.visible_authoritative_entity_records_materialized >= 2);
    assert!(snapshot_counters.visible_authoritative_relation_records_materialized >= 1);

    runtime.performance_access().reset_counters();
    let _ = runtime.read_truth().read_version(historical_version);
    let historical_version_counters = runtime.performance_access().counters();

    assert_eq!(historical_version_counters.visibility_entity_slot_scans, 2);
    assert_eq!(
        historical_version_counters.visibility_relation_slot_scans,
        1
    );
    assert_eq!(
        historical_version_counters.visibility_cache_miss_reconstructions,
        1
    );
    assert_eq!(
        historical_version_counters.visibility_exact_state_materializations,
        0
    );
    assert!(historical_version_counters.visible_authoritative_entity_records_materialized >= 2);
    assert!(historical_version_counters.visible_authoritative_relation_records_materialized >= 1);

    runtime.performance_access().reset_counters();
    let _ = runtime.read_truth().read_version(current_version);
    let current_version_counters = runtime.performance_access().counters();

    assert_eq!(current_version_counters.visibility_entity_slot_scans, 0);
    assert_eq!(current_version_counters.visibility_relation_slot_scans, 0);
    assert_eq!(
        current_version_counters.visibility_cache_miss_reconstructions,
        1
    );
    assert_eq!(
        current_version_counters.visibility_exact_state_materializations,
        0
    );
    assert_eq!(current_version_counters.visibility_cache_hits, 0);
}
