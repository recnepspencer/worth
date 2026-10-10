use super::*;

#[test]
fn historical_reader_from_live_head_survives_head_retirement() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let anchor = create_entity_outcome(&runtime, "historical-reader-anchor");
    let branch = create_branch_from_main(&runtime, "historical-reader-fork");
    let first =
        create_entity_outcome_on_branch(&runtime, "historical-reader-first", branch.clone());
    let index = scoped_name_index(&runtime);
    let first_generation =
        build_on_branch(execution, &runtime, index.index_id, &first, branch.clone());
    release_test_commit_snapshot(&runtime, &anchor);
    release_test_commit_snapshot(&runtime, &first);
    let advanced_main = create_entity_outcome(&runtime, "historical-reader-main-advanced");
    release_test_commit_snapshot(&runtime, &advanced_main);
    let retained = crate::visibility::snapshot_states::HistoricalVisibilityBasis::resolve(
        &runtime,
        first.commit.version_id,
    )
    .unwrap();
    assert_eq!(retained.branch_id(), &branch);
    let next = create_entity_outcome_on_branch(&runtime, "historical-reader-next", branch.clone());
    let next_generation = build_on_branch(execution, &runtime, index.index_id, &next, branch);
    release_test_commit_snapshot(&runtime, &next);
    assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(0));
    assert!(runtime.indexes.generation(first_generation).is_some());
    assert!(runtime.indexes.generation(next_generation).is_some());
    drop(retained);
    assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(1));
    assert!(runtime.indexes.generation(first_generation).is_none());
    assert!(runtime.indexes.generation(next_generation).is_some());
}
