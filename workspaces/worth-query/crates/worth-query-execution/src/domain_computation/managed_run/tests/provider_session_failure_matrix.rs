use std::sync::Arc;

use super::provider_session_protocol::{
    cleanup, session_run, staged_session, SessionCallCounts, SessionCallObservation,
    SessionFailurePoint,
};
use crate::domain_computation::{
    WorthQueryProviderSessionProtocolStage, WorthQueryProviderSessionRecoveryPosture,
};

#[test]
fn readmission_and_preparation_failures_stop_at_the_exact_callback() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        assert_early_failure(
            execution,
            SessionFailurePoint::ReadmissionRejection,
            WorthQueryProviderSessionProtocolStage::PlanReadmission,
            WorthQueryProviderSessionRecoveryPosture::Closed,
            calls(1, 0, 0, 0, 0),
        );
        assert_early_failure(
            execution,
            SessionFailurePoint::ReadmissionPanic,
            WorthQueryProviderSessionProtocolStage::PlanReadmission,
            WorthQueryProviderSessionRecoveryPosture::RecoveryRequired,
            calls(1, 0, 0, 0, 0),
        );
        assert_early_failure(
            execution,
            SessionFailurePoint::PreparationRejection,
            WorthQueryProviderSessionProtocolStage::SessionPreparation,
            WorthQueryProviderSessionRecoveryPosture::Closed,
            calls(1, 1, 0, 0, 1),
        );
        assert_early_failure(
            execution,
            SessionFailurePoint::PreparationPanic,
            WorthQueryProviderSessionProtocolStage::SessionPreparation,
            WorthQueryProviderSessionRecoveryPosture::Closed,
            calls(1, 1, 0, 0, 1),
        );
    });
}

#[test]
fn staged_preparation_failures_abort_without_reaching_commit() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        for point in [
            SessionFailurePoint::StagedPreparationRejection,
            SessionFailurePoint::StagedPreparationPanic,
        ] {
            let observed = with_session(point, |running, graph| {
                let failure = staged_session(execution, running, graph)
                    .prepare_for_commit()
                    .expect_err("staged preparation failure must not mint a prepare outcome");
                assert_failure(
                    &failure,
                    WorthQueryProviderSessionProtocolStage::StagedPreparation,
                    WorthQueryProviderSessionRecoveryPosture::Closed,
                );
            });
            assert_eq!(observed, calls(1, 1, 1, 0, 1));
        }
    });
}

#[test]
fn commit_failures_leave_recovery_required_and_drop_aborts_once() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        for point in [
            SessionFailurePoint::CommitRejection,
            SessionFailurePoint::CommitPanic,
        ] {
            let observed = with_session(point, |running, graph| {
                let prepared = staged_session(execution, running, graph)
                    .prepare_for_commit()
                    .expect("staged preparation should succeed");
                let outcome = prepared.commit();
                assert_failure(
                    outcome.failure().expect("commit should fail"),
                    WorthQueryProviderSessionProtocolStage::Commit,
                    WorthQueryProviderSessionRecoveryPosture::RecoveryRequired,
                );
            });
            assert_eq!(observed, calls(1, 1, 1, 1, 1));
        }
    });
}

#[test]
fn abort_failures_are_retried_once_by_the_live_session_guard() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        for point in [
            SessionFailurePoint::AbortRejection,
            SessionFailurePoint::AbortPanic,
        ] {
            let observed = with_session(point, |running, graph| {
                let outcome = staged_session(execution, running, graph).abort();
                assert_failure(
                    outcome.failure().expect("abort should fail"),
                    WorthQueryProviderSessionProtocolStage::Abort,
                    WorthQueryProviderSessionRecoveryPosture::RecoveryRequired,
                );
            });
            assert_eq!(observed, calls(1, 1, 0, 0, 2));
        }
    });
}

fn assert_early_failure(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    point: SessionFailurePoint,
    stage: WorthQueryProviderSessionProtocolStage,
    posture: WorthQueryProviderSessionRecoveryPosture,
    expected_calls: SessionCallObservation,
) {
    let observed = with_session(point, |running, graph| {
        let plan = running
            .admit_provider_execution_plan(graph)
            .expect("failure fixture plan should admit");
        let failure = match point {
            SessionFailurePoint::ReadmissionRejection | SessionFailurePoint::ReadmissionPanic => {
                plan.readmit(execution)
                    .expect_err("readmission should fail")
            }
            SessionFailurePoint::PreparationRejection | SessionFailurePoint::PreparationPanic => {
                plan.readmit(execution)
                    .expect("readmission should succeed")
                    .prepare()
                    .expect_err("preparation should fail")
            }
            _ => unreachable!("early failure helper received a later phase"),
        };
        assert_failure(&failure, stage, posture);
    });
    assert_eq!(observed, expected_calls);
}

fn with_session(
    point: SessionFailurePoint,
    assertion: impl FnOnce(
        &mut crate::domain_computation::WorthQueryRunningDirectRun,
        &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    ),
) -> SessionCallObservation {
    let counts = Arc::new(SessionCallCounts::default());
    let (mut running, graph) = session_run(point, Arc::clone(&counts), false);
    assertion(&mut running, &graph);
    cleanup(running);
    counts.observe()
}

fn assert_failure(
    failure: &crate::domain_computation::WorthQueryProviderSessionFailure,
    stage: WorthQueryProviderSessionProtocolStage,
    posture: WorthQueryProviderSessionRecoveryPosture,
) {
    assert_eq!(failure.stage(), stage);
    assert_eq!(failure.recovery_posture(), posture);
}

const fn calls(
    readmissions: usize,
    preparations: usize,
    staged_preparations: usize,
    commits: usize,
    aborts: usize,
) -> SessionCallObservation {
    SessionCallObservation {
        readmissions,
        preparations,
        staged_preparations,
        commits,
        aborts,
    }
}
