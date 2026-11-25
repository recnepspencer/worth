//! Fixture adaptation (C2.5): restores f9f15bf8c8 historical reader survival
//! and sibling/pinned derived-index reclamation over the Memory runtime (no retired test medium).
use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

macro_rules! scoped_name_index {
    ($runtime:expr) => {
        $runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "scoped-checkpoint-index".into(),
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                },
                branch_scoped: true,
            })
            .expect("index fixture has identity capacity")
    };
}

macro_rules! build_on_branch {
    ($execution:expr, $runtime:expr, $index:expr, $outcome:expr, $branch:expr) => {{
        let built = $runtime.index_authority().build_for_commit(
            $execution,
            DerivedIndexBuildRequest {
                source_commit_id: $outcome.commit.commit_id,
                branch_id: $branch,
                index_ids: vec![$index],
            },
        );
        assert!(built.failed_indexes.is_empty());
        built.generations[0].generation_id
    }};
}

#[test]
fn historical_reader_from_live_head_survives_head_retirement() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "historical-reader-anchor");
    let branch = create_branch_from_main(&runtime, "historical-reader-fork");
    let first = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "historical-reader-first",
        branch.clone(),
    );
    release_test_commit_snapshot(&runtime, &anchor);
    release_test_commit_snapshot(&runtime, &first);
    let advanced_main =
        create_entity_outcome(execution, &runtime, "historical-reader-main-advanced");
    release_test_commit_snapshot(&runtime, &advanced_main);
    let index = scoped_name_index!(runtime);
    let first_generation =
        build_on_branch!(execution, runtime, index.index_id, &first, branch.clone());
    let retained = crate::visibility::snapshot_states::HistoricalVisibilityBasis::resolve(
        &runtime,
        first.commit.version_id,
    )
    .unwrap();
    assert_eq!(retained.branch_id(), &branch);
    let next = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "historical-reader-next",
        branch.clone(),
    );
    let next_generation = runtime
        .index_access()
        .read_latest_generation(execution, index.index_id, &branch)
        .unwrap()
        .expect("branch publication maintains the registered scoped index generation")
        .generation_id;
    release_test_commit_snapshot(&runtime, &next);
    assert_eq!(
        runtime
            .run_index_generation_reclamation_after_publication(&next)
            .unwrap(),
        Some(0)
    );
    assert!(runtime.indexes.generation(first_generation).is_some());
    assert!(runtime.indexes.generation(next_generation).is_some());
    drop(retained);
    assert_eq!(
        runtime
            .run_index_generation_reclamation_after_publication(&next)
            .unwrap(),
        Some(1)
    );
    assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(0));
    assert!(runtime.indexes.generation(first_generation).is_none());
    assert!(runtime.indexes.generation(next_generation).is_some());
}

#[test]
fn reclamation_and_checkpoint_keep_sibling_head_generation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "sibling-index-first");
    let sibling = create_branch_from_main(&runtime, "index-sibling");
    let index = name_index(&runtime);
    let _first_generation = build(execution, &runtime, index.index_id, &first);
    let sibling_commit = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "sibling-index-own-head",
        sibling.clone(),
    );
    let sibling_generation = build_on_branch!(
        execution,
        runtime,
        index.index_id,
        &sibling_commit,
        sibling.clone()
    );
    let main = create_entity_outcome(execution, &runtime, "main-index-new-head");
    let main_generation = build(execution, &runtime, index.index_id, &main);
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &sibling_commit);
    release_test_commit_snapshot(&runtime, &main);
    runtime.run_branch_root_reclamation_pass();
    runtime.run_index_generation_reclamation_pass();
    assert!(runtime.indexes.generation(sibling_generation).is_some());
    assert!(runtime.indexes.generation(main_generation).is_some());
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert!(recovered.indexes.generation(sibling_generation).is_some());
    assert!(recovered.indexes.generation(main_generation).is_some());
}

#[test]
fn reclamation_keeps_a_pinned_old_reader_until_release() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "pinned-index-first");
    let index = name_index(&runtime);
    let first_generation = build(execution, &runtime, index.index_id, &first);
    let middle = create_entity_outcome(execution, &runtime, "pinned-index-middle");
    let middle_generation = build(execution, &runtime, index.index_id, &middle);
    let latest = create_entity_outcome(execution, &runtime, "pinned-index-latest");
    let latest_generation = build(execution, &runtime, index.index_id, &latest);
    release_test_commit_snapshot(&runtime, &middle);
    release_test_commit_snapshot(&runtime, &latest);
    runtime.run_branch_root_reclamation_pass();
    assert!(runtime.indexes.generation(middle_generation).is_some());
    runtime.run_index_generation_reclamation_pass();
    assert!(runtime.indexes.generation(first_generation).is_some());
    assert!(runtime.indexes.generation(latest_generation).is_some());
    assert!(runtime.indexes.generation(middle_generation).is_none());
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert!(recovered.indexes.generation(first_generation).is_none());
    assert!(recovered.indexes.generation(latest_generation).is_some());
    release_test_commit_snapshot(&runtime, &first);
    runtime.run_branch_root_reclamation_pass();
    runtime.run_index_generation_reclamation_pass();
    assert!(runtime.indexes.generation(first_generation).is_none());
    assert!(runtime.indexes.generation(latest_generation).is_some());
}
