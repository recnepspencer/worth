use super::certification_bundle::run_merge_ready_history_shape_certification;

#[test]
fn merge_ready_history_ancestry_queries_resolve_branch_reasoning() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let certification = run_merge_ready_history_shape_certification(execution);
    let parent_lists = &certification.parent_list_serialization;
    let ancestry = &certification.ancestry_query_matrix;

    assert_eq!(
        ancestry.pre_merge_common_ancestor_commit_id,
        Some(parent_lists.linear_commit_id)
    );
    assert_eq!(
        ancestry.post_merge_common_ancestor_commit_id,
        Some(parent_lists.feature_commit_id)
    );
    assert_eq!(
        ancestry.inspected_merge_base_commit_id,
        Some(parent_lists.feature_commit_id)
    );
    assert_eq!(
        ancestry.merge_ready_commit_ancestor_closure,
        vec![
            parent_lists.root_commit_id,
            parent_lists.linear_commit_id,
            parent_lists.feature_commit_id,
            parent_lists.merge_ready_commit_id,
        ]
    );
    assert_eq!(
        ancestry.main_head_commit_id,
        *ancestry.main_head_ancestor_closure.last().unwrap()
    );
    assert_ne!(
        ancestry.main_head_ancestor_closure,
        ancestry.merge_ready_commit_ancestor_closure
    );
    assert!(certification.branch_reasoning.inspected_merge_base_present);
    assert!(
        certification
            .branch_reasoning
            .main_head_closure_contains_head
    );
}
