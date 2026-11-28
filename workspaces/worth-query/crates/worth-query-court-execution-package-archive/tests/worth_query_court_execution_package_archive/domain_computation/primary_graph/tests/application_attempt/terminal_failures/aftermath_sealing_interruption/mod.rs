//! The commit door preserves a request that stops while sealing aftermath.
use crate::domain_computation::primary_graph::tests::application_attempt::terminal_failures::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[test]
fn native_preparation_control_stop_keeps_failed_cleanup_evidence_at_the_commit_door() {
    let _listener =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    for expired in [false, true] {
        let time = worth_execution::MonotonicTimeController::new().unwrap();
        let held = time.hold();
        let world = installed_authorization_world(true, time.clock());
        let source = WorthQueryCancellationSource::new();
        let deadline = expired.then(|| held + std::time::Duration::from_secs(2));
        let request = WorthQueryRequestScope::new(deadline, source.token(), time.clock());
        let setup = request.clone();
        let program = world
            .application
            .with_host_advancement(|phase| {
                let principal =
                    authenticated_principal(&phase.test_host_request_for_test(), &world, &setup);
                let account =
                    resolved_account(&phase.test_host_request_for_test(), &world, "open", &setup);
                admitted_program(
                    &phase,
                    &world,
                    &principal,
                    &account,
                    &setup,
                    "direct-control-cleanup",
                )
            })
            .unwrap();
        let interruption = world.faults.interrupt_next_native_preparation_for_test(
            source,
            deadline.map(|instant| (time.clone(), instant)),
        );
        let _cleanup = world
            .application
            .schedule_next_managed_cleanup_on_worker_for_test()
            .unwrap();
        let outcome = world
            .application
            .with_application_advancement(&request, |phase| {
                world
                    .application
                    .compare_and_commit_application_in_advancement(
                        &phase,
                        program,
                        idempotency(172, 172),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
            })
            .unwrap();
        assert!(
            interruption.reached_for_test(),
            "stop occurs inside native preparation"
        );
        let outcome = discharge_actual_cleanup(&world, outcome);
        let WorthQueryApplicationCommitOutcome::Indeterminate(evidence) = outcome else {
            panic!("stopped progression and failed cleanup retain evidence: {outcome:?}")
        };
        use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitRecoveryKind as Recovery;
        use worth_query_execution::facade::domain_computation::WorthQueryApplicationUnresolvedCommitEvidence as Evidence;
        let stop = if expired {
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut
        } else {
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
        };
        let failure =
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure::checked_read_stopped_for_test(
                Err(stop),
                worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
                "managed mutation run failed to finish after provider progression",
            );
        assert_eq!(
            evidence,
            Evidence::from_provider_session_failure_for_test(
                Recovery::CommitRecoveryRequired,
                &failure
            )
            .with_cleanup_failure_for_test()
        );
    }
}

pub fn discharge_actual_cleanup(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    outcome: WorthQueryApplicationCommitOutcome,
) -> WorthQueryApplicationCommitOutcome {
    let WorthQueryApplicationCommitOutcome::CleanupRequired(required) = outcome else {
        return outcome;
    };
    let (outcome, cleanup) = required.into_parts();
    assert_eq!(cleanup.failure().kind(), worth_query_execution::facade::domain_computation::WorthQueryMutationCleanupFailureKind::Native(
        worth_query_execution::facade::domain_computation::WorthQueryManagedRunCleanupFailureKind::BridgeFinalization(
            worth_runtime_bridge::facade::BridgeExecutionBasisFinalizationFailureKind::SignalRuntimeThreadAffinityViolation)));
    let scope = live_scope();
    let receipt = world
        .application
        .with_application_advancement(&scope, |phase| cleanup.retry_in_advancement(&phase))
        .unwrap()
        .unwrap();
    assert!(receipt.inspection().resources_released());
    outcome
}
