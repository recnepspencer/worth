use super::*;

#[test]
fn expired_candidate_summary_is_typed_and_cannot_retain_stale_scope() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(test_schema_registry())
        .publication(crate::facade::config::PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4_096,
            max_published_snapshot_handles: 1,
            max_active_snapshot_handles: 8,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 1,
            candidate_max_lifetime_millis: 0,
        })
        .build();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("expired-summary"),
            Allocation::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction, Allocation::SystemAllocation)
        .unwrap();
    assert_eq!(
        runtime
            .preparation_port()
            .summarize_prepared_candidate(&candidate, GENEROUS),
        Err(Denial::CandidateLifetimeExpired)
    );
    runtime.discard_prepared_candidate(candidate).unwrap();
}
