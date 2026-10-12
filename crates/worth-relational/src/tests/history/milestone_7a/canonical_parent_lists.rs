use super::certification_bundle::run_merge_ready_history_shape_certification;

#[test]
fn merge_ready_history_parent_lists_survive_publication_replay_and_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let certification = run_merge_ready_history_shape_certification(execution);
    let parent_lists = &certification.parent_list_serialization;

    assert_eq!(parent_lists.root_parents, Vec::<u64>::new());
    assert_eq!(
        parent_lists.linear_parents,
        vec![parent_lists.root_commit_id]
    );
    assert_eq!(
        parent_lists.feature_parents,
        vec![parent_lists.linear_commit_id]
    );
    assert_eq!(
        parent_lists.merge_ready_parents,
        vec![
            parent_lists.linear_commit_id,
            parent_lists.feature_commit_id,
        ]
    );
    assert_eq!(
        parent_lists.replayed_merge_ready_parents,
        parent_lists.merge_ready_parents
    );
    assert_eq!(
        parent_lists.recovered_merge_ready_parents,
        parent_lists.merge_ready_parents
    );
}
