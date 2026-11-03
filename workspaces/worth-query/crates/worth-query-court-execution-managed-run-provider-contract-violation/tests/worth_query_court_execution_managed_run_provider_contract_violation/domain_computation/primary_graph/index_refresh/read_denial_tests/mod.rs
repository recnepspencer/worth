//! Translation preserves observed committed facts; it makes no publication claim.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::index_refresh::patch_read_denial;
use worth_query_execution::facade::integration::WorthQueryPrimaryGraphIndexRefreshDenialKind;

#[test]
fn patch_read_failure_preserves_storage_cause_and_observed_committed_context() {
    use worth_relational::facade::publication::PatchStreamReadErrorClass;
    use worth_relational::facade::storage::RelationalStorageSubject;
    use worth_relational::facade::storage::RelationalStorageUnavailable;
    use worth_relational::facade::storage::RelationalStorageUnavailableCause;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause;
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let graph = &world.application.primary_provider.graph;
    let (commit, position) = graph.with_runtime(execution, |runtime| {
        (
            runtime
                .history()
                .historical_latest_commit()
                .expect("fixture has a genuine bootstrap commit"),
            runtime.history().latest_patch_stream_position().unwrap(),
        )
    });
    let failure = RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Envelope(position),
        cause: RelationalStorageUnavailableCause::SourceUnavailable,
    };
    let class = PatchStreamReadErrorClass::ReadRequest(
        RelationalExecutionDenialCause::StorageUnavailable(failure),
    );
    let requested = graph.primary_index_ids.len();
    let refusal = patch_read_denial(None, Some(&commit), requested, class);
    assert_eq!(
        refusal.kind(),
        WorthQueryPrimaryGraphIndexRefreshDenialKind::Execution(Ok(
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind::StorageUnavailable(
                failure
            ),
        ))
    );
    assert_eq!(refusal.committed_mutation_id(), Some(commit.commit_id));
    assert_eq!(refusal.committed_branch_id(), Some(&commit.branch_id));
    assert_eq!(refusal.failed_index_count(), requested);
    let unchanged = patch_read_denial(Some(&commit), Some(&commit), requested, class);
    assert_eq!(unchanged.previous_commit_id(), Some(commit.commit_id));
    assert_eq!(unchanged.committed_mutation_id(), None);
    assert_eq!(unchanged.kind(), refusal.kind());
}
