use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progressed::association_tests::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;

#[test]
fn genuinely_interleaved_equivalent_sessions_validate_retry_cleanup_separately() {
    let world = installed_authorization_world(true);
    world
        .application
        .with_application_advancement(
            &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope(),
            |active_phase| {
                let host_custody = active_phase.test_host_request_for_test();
                let phase = &active_phase;
                let snapshot_baseline = world
                    .invariant
                    .active_snapshot_count_for_test(&active_phase.borrowed_request_for_test());
                let (left, right) = equivalent_programs(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    "racing-equivalent",
                );
                let binding = idempotency(139, 140);
                let left = start(phase, &world.application, left, binding);
                let right = start(phase, &world.application, right, binding);
                let left = progress_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    left,
                    None,
                    ExecutionAllocationPolicy::SystemAllocation,
                );
                let right = progress_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    right,
                    None,
                    ExecutionAllocationPolicy::SystemAllocation,
                );
                assert_eq!(
                    world
                        .invariant
                        .active_snapshot_count_for_test(&active_phase.borrowed_request_for_test()),
                    snapshot_baseline + 2
                );
                let left = finish_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    left,
                );
                assert_eq!(
                    world
                        .invariant
                        .active_snapshot_count_for_test(&active_phase.borrowed_request_for_test()),
                    snapshot_baseline + 1,
                    "finishing one attempt must preserve its interleaved peer's snapshot lease"
                );
                let right = finish_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    right,
                );
                assert_eq!(
                    world
                        .invariant
                        .active_snapshot_count_for_test(&active_phase.borrowed_request_for_test()),
                    snapshot_baseline
                );
                assert_eq!(
                    world.application.provider_session_resource_count_for_test(),
                    0
                );

                let (executed, recovered) = match (left, right) {
                    (
                        WorthQueryApplicationCommitOutcome::Committed(executed),
                        WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered),
                    )
                    | (
                        WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered),
                        WorthQueryApplicationCommitOutcome::Committed(executed),
                    ) => (executed, recovered),
                    unexpected => {
                        panic!("expected one executed and one recovered commit: {unexpected:?}")
                    }
                };
                assert!(executed.is_same_authoritative_commit(&recovered));
                assert_eq!(executed.retained_preimage_for_test(), recovered.retained_preimage_for_test());
                assert_eq!(executed.dispatch_outbox(), recovered.dispatch_outbox());
                let executed_output = executed
                    .outputs_of::<RetentionOutputs>()
                    .and_then(|outputs| outputs.entity::<RetainedAccount>())
                    .expect("fresh receipt must retain the bound output role");
                let recovered_output = recovered
                    .outputs_of::<RetentionOutputs>()
                    .and_then(|outputs| outputs.entity::<RetainedAccount>())
                    .expect("idempotent receipt must recover the same output role");
                assert_eq!(executed_output.entity_id(), recovered_output.entity_id());
                assert_eq!(executed.terminal().attempt_resources_released(), Some(true));
                assert_eq!(
                    recovered.terminal().attempt_resources_released(),
                    Some(true)
                );
                assert_eq!(
                    executed.terminal().kind(),
                    WorthQueryApplicationCommitTerminalKind::Executed
                );
                assert_eq!(
                    recovered.terminal().kind(),
                    WorthQueryApplicationCommitTerminalKind::Recovered
                );
            },
        )
        .unwrap();
}
