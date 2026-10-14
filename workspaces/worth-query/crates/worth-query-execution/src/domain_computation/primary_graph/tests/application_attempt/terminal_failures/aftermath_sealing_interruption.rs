//! The commit door preserves a request that stops while sealing aftermath.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn aftermath_sealing_interruption_reaches_commit_evidence() {
    let _listener =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for (expired, reject_cleanup) in [(false, false), (true, false), (false, true), (true, true)] {
        let world = installed_authorization_world(true);
        let request = live_scope();
        let (program, parent) = world
            .application
            .with_host_advancement(|phase| {
                let principal =
                    authenticated_principal(&phase.test_host_request(), &world, &request);
                let account =
                    resolved_account(&phase.test_host_request(), &world, "open", &request);
                let program = admitted_program(
                    &phase,
                    &world,
                    &principal,
                    &account,
                    &request,
                    "aftermath-stop",
                );
                let parent = world
                    .selected_product(&phase.test_host_request())
                    .product()
                    .relational_basis()
                    .observation()
                    .commit_receipt()
                    .cloned()
                    .expect("fixture has its parent receipt");
                (program, parent)
            })
            .unwrap();
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
                world
                    .application
                    .compare_and_commit_application_with_aftermath(
                        &phase,
                        program,
                        idempotency(171, 171),
                        WorthQueryPendingAftermathCausality::undo_of(parent),
                        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                    )
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
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut
        } else {
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
        };
        assert_eq!(evidence.denial_kind(), Err(expected));
        assert_eq!(evidence.recovery(), crate::domain_computation::primary_graph::WorthQueryApplicationCommitRecoveryKind::CommitRecoveryRequired);
        assert_eq!(
            evidence.stage(),
            crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit
        );
        use crate::domain_computation::primary_graph::{
            WorthQueryApplicationCommitRecoveryKind as Recovery,
            WorthQueryApplicationManagedCleanupFailure as Cleanup,
            WorthQueryApplicationUnresolvedCommitEvidence as Evidence,
        };
        let expected_failure =
            crate::domain_computation::WorthQueryProviderSessionFailure::checked_read_stopped(
                Err(expected),
                crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
                "request stopped during aftermath sealing",
            );
        let mut expected_evidence = Evidence::from_provider_session_failure(
            Recovery::CommitRecoveryRequired,
            &expected_failure,
        );
        if reject_cleanup {
            expected_evidence = expected_evidence.with_cleanup_failure();
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
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
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
                let principal = authenticated_principal(&phase.test_host_request(), &world, &setup);
                let account = resolved_account(&phase.test_host_request(), &world, "open", &setup);
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
                        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
        use crate::domain_computation::primary_graph::{
            WorthQueryApplicationCommitRecoveryKind as Recovery,
            WorthQueryApplicationUnresolvedCommitEvidence as Evidence,
        };
        let stop = if expired {
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut
        } else {
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
        };
        let failure =
            crate::domain_computation::WorthQueryProviderSessionFailure::checked_read_stopped(
                Err(stop),
                crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
                "managed mutation run failed to finish after provider progression",
            );
        assert_eq!(
            evidence,
            Evidence::from_provider_session_failure(Recovery::CommitRecoveryRequired, &failure)
                .with_cleanup_failure()
        );
    }
}

fn discharge_actual_cleanup(
    world: &crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld,
    outcome: WorthQueryApplicationCommitOutcome,
) -> WorthQueryApplicationCommitOutcome {
    let WorthQueryApplicationCommitOutcome::CleanupRequired(required) = outcome else {
        return outcome;
    };
    let (outcome, cleanup) = required.into_parts();
    assert_eq!(cleanup.failure().kind(), crate::domain_computation::WorthQueryMutationCleanupFailureKind::Native(
        crate::domain_computation::WorthQueryManagedRunCleanupFailureKind::BridgeFinalization(
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
