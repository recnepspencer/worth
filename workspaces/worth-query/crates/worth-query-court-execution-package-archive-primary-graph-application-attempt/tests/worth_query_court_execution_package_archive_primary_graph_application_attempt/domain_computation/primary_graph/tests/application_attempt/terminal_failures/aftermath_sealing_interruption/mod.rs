//! The commit door preserves a request that stops while sealing aftermath.
use crate::domain_computation::primary_graph::tests::application_attempt::terminal_failures::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[test]
fn aftermath_sealing_interruption_reaches_commit_evidence() {
    let _listener =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    for (expired, reject_cleanup) in [(false, false), (true, false), (false, true), (true, true)] {
        let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::install();
        let source = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::edit(&world, "sealing-source", "edited-label");
        let prepared = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::prepare(&world, &source, "sealing-correction");
        let cancellation = WorthQueryCancellationSource::new();
        let deadline =
            expired.then(|| std::time::Instant::now() + std::time::Duration::from_secs(2));
        let request = WorthQueryRequestScope::new(deadline, cancellation.token());
        world
            .faults
            .interrupt_next_aftermath_seal(cancellation, deadline);
        let _cleanup_guard = reject_cleanup.then(|| {
            world
                .application
                .schedule_next_managed_cleanup_on_worker_for_test()
                .unwrap()
        });
        let outcome = world
            .application
            .with_application_advancement(&request, |phase| {
                worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::correction_source::publish_in_advancement(&world, &phase, prepared)
            })
            .unwrap();
        let outcome = discharge_actual_cleanup(&world, outcome);
        let WorthQueryApplicationCommitOutcome::Indeterminate(evidence) = outcome else {
            panic!("sealing stop must retain unresolved commit evidence: {outcome:?}");
        };
        assert_eq!(world.faults.aftermath_interruption_count(), 1);
        assert_eq!(
            evidence.detail(),
            "request stopped during aftermath sealing"
        );
        let expected = if expired {
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut
        } else {
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
        };
        assert_eq!(evidence.denial_kind(), Err(expected));
        assert_eq!(evidence.recovery(), worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitRecoveryKind::CommitRecoveryRequired);
        assert_eq!(
            evidence.stage(),
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolStage::Commit
        );
        use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitRecoveryKind as Recovery;
        use worth_query_execution::facade::domain_computation::WorthQueryApplicationManagedCleanupFailure as Cleanup;
        use worth_query_execution::facade::domain_computation::WorthQueryApplicationUnresolvedCommitEvidence as Evidence;
        let expected_failure =
            worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure::checked_read_stopped_for_test(
                Err(expected),
                worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
                "request stopped during aftermath sealing",
            );
        let mut expected_evidence = Evidence::from_provider_session_failure_for_test(
            Recovery::CommitRecoveryRequired,
            &expected_failure,
        );
        if reject_cleanup {
            expected_evidence = expected_evidence.with_cleanup_failure_for_test();
        }
        assert_eq!(
            evidence.cleanup_failure(),
            reject_cleanup.then_some(Cleanup::RunFinishRejected)
        );
        assert_eq!(evidence, expected_evidence);
    }
}

#[test]
fn native_preparation_control_stop_keeps_failed_cleanup_evidence_at_the_commit_door() {
    let _listener =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    for expired in [false, true] {
        let world = installed_authorization_world(true);
        let source = WorthQueryCancellationSource::new();
        let deadline =
            expired.then(|| std::time::Instant::now() + std::time::Duration::from_secs(2));
        let request = WorthQueryRequestScope::new(deadline, source.token());
        let setup = request.clone();
        let program = world
            .application
            .with_host_advancement(|phase| {
                let principal = authenticated_principal(&phase.test_host_request_for_test(), &world, &setup);
                let account = resolved_account(&phase.test_host_request_for_test(), &world, "open", &setup);
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
        let interruption = world
            .faults
            .interrupt_next_native_preparation_for_test(source, deadline);
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
            Evidence::from_provider_session_failure_for_test(Recovery::CommitRecoveryRequired, &failure)
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
    outcome.into()
}
