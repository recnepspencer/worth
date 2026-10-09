use std::sync::Arc;

use super::provider_session_protocol::{
    cleanup, session_run, SessionCallCounts, SessionCallObservation, SessionFailurePoint,
};

#[test]
fn dropping_a_readmitted_session_aborts_once() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let calls = Arc::new(SessionCallCounts::default());
        let (mut running, graph) =
            session_run(SessionFailurePoint::None, Arc::clone(&calls), false);
        drop(
            running
                .admit_provider_execution_plan(&graph)
                .expect("provider plan should admit")
                .readmit(execution)
                .expect("provider plan should readmit"),
        );
        assert_eq!(
            calls.observe(),
            SessionCallObservation {
                readmissions: 1,
                aborts: 1,
                ..SessionCallObservation::default()
            }
        );
        cleanup(running);
    });
}

#[test]
fn dropping_a_prepared_session_aborts_once() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let calls = Arc::new(SessionCallCounts::default());
        let (mut running, graph) =
            session_run(SessionFailurePoint::None, Arc::clone(&calls), false);
        drop(
            running
                .admit_provider_execution_plan(&graph)
                .expect("provider plan should admit")
                .readmit(execution)
                .expect("provider plan should readmit")
                .prepare()
                .expect("provider session should prepare"),
        );
        assert_eq!(
            calls.observe(),
            SessionCallObservation {
                readmissions: 1,
                preparations: 1,
                aborts: 1,
                ..SessionCallObservation::default()
            }
        );
        cleanup(running);
    });
}

#[test]
fn dropping_a_staged_session_aborts_once() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let calls = Arc::new(SessionCallCounts::default());
        let (mut running, graph) =
            session_run(SessionFailurePoint::None, Arc::clone(&calls), false);
        drop(
            running
                .admit_provider_execution_plan(&graph)
                .expect("provider plan should admit")
                .readmit(execution)
                .expect("provider plan should readmit")
                .prepare()
                .expect("provider session should prepare")
                .bind_reads_and_effects(),
        );
        assert_eq!(
            calls.observe(),
            SessionCallObservation {
                readmissions: 1,
                preparations: 1,
                aborts: 1,
                ..SessionCallObservation::default()
            }
        );
        cleanup(running);
    });
}

#[test]
fn dropping_a_commit_prepared_session_aborts_once() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let calls = Arc::new(SessionCallCounts::default());
        let (mut running, graph) =
            session_run(SessionFailurePoint::None, Arc::clone(&calls), false);
        drop(
            running
                .admit_provider_execution_plan(&graph)
                .expect("provider plan should admit")
                .readmit(execution)
                .expect("provider plan should readmit")
                .prepare()
                .expect("provider session should prepare")
                .bind_reads_and_effects()
                .prepare_for_commit()
                .expect("staged session should prepare for commit"),
        );
        assert_eq!(
            calls.observe(),
            SessionCallObservation {
                readmissions: 1,
                preparations: 1,
                staged_preparations: 1,
                aborts: 1,
                ..SessionCallObservation::default()
            }
        );
        cleanup(running);
    });
}

#[test]
fn failed_preparation_aborts_once() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let calls = Arc::new(SessionCallCounts::default());
        let (mut running, graph) = session_run(
            SessionFailurePoint::PreparationPanic,
            Arc::clone(&calls),
            false,
        );
        let _ = running
            .admit_provider_execution_plan(&graph)
            .expect("provider plan should admit")
            .readmit(execution)
            .expect("provider plan should readmit")
            .prepare()
            .expect_err("preparation panic must be contained");
        assert_eq!(
            calls.observe(),
            SessionCallObservation {
                readmissions: 1,
                preparations: 1,
                aborts: 1,
                ..SessionCallObservation::default()
            }
        );
        cleanup(running);
    });
}
