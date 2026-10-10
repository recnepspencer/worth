use super::fixture::{
    direct_admission_fixture, workflow_admission_fixture, workflow_epoch_fixture,
    DirectAdmissionFixture, FixtureDisposition, WorkflowAdmissionFixture, WORKFLOW_STAGE,
};
use super::terminal_fixture::{converged_terminal, workflow_converged_terminal};
use crate::domain_computation::{
    WorthQueryConverged, WorthQueryDirectConvergenceIterationOutcome,
    WorthQueryDirectConvergenceReadmissionOutcome, WorthQueryDirectConvergenceStepOutcome,
    WorthQueryDirectConvergenceYieldOutcome, WorthQueryGraphProviderCallKind,
    WorthQueryManagedGraphCallRequest, WorthQueryReadmissionEvidence,
    WorthQueryWorkflowConvergenceCleanupOutcome, WorthQueryWorkflowConvergenceIterationOutcome,
    WorthQueryWorkflowConvergenceReadmissionOutcome, WorthQueryWorkflowConvergenceStepOutcome,
    WorthQueryWorkflowConvergenceTerminal, WorthQueryWorkflowConvergenceYieldOutcome,
};

#[test]
fn same_runtime_yield_and_readmission_preserve_the_semantic_convergence_result() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let ordinary = converged_terminal(execution, resource_request);
        let DirectAdmissionFixture {
            runtime,
            operation,
            alternate_basis_operation: _,
            contract,
            managed,
            graph,
            bridge,
        } = direct_admission_fixture(FixtureDisposition::YieldThenConverged, resource_request);
        let epoch =
            match runtime.admit_direct_convergence_epoch(&operation, contract, managed, graph) {
                Ok(epoch) => epoch.start(),
                Err(_) => panic!("yield convergence authorities must admit"),
            };
        let started = match epoch.begin_iteration(
            execution,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "yielded-convergence-iteration",
            ),
        ) {
            Ok(started) => started,
            Err(_) => panic!("yield convergence iteration must start"),
        };
        let paused = match started.advance(execution) {
            WorthQueryDirectConvergenceStepOutcome::Continue(paused) => paused,
            _ => panic!("yield provider must expose its installed safe point"),
        };
        let yielded = match paused.yield_iteration() {
            WorthQueryDirectConvergenceYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("eligible convergence iteration must preserve yielded authority"),
        };
        let resumed = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            WorthQueryDirectConvergenceReadmissionOutcome::Readmitted(readmitted) => {
                assert_committed_readmission_evidence(readmitted.readmission_evidence());
                readmitted.into_started()
            }
            _ => panic!("same-runtime convergence readmission must restore the provider"),
        };
        let outcome = match resumed.advance(execution) {
            WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
            _ => panic!("restored convergence provider must complete and rejoin its epoch"),
        };
        let resumed = match outcome {
            WorthQueryDirectConvergenceIterationOutcome::Converged(terminal) => terminal,
            _ => panic!("restored installed comparator must converge"),
        };

        assert_eq!(ordinary.kind(), resumed.kind());
        assert_eq!(
            ordinary.latest_report().unwrap().decision(),
            resumed.latest_report().unwrap().decision()
        );
        assert_eq!(
            ordinary.latest_report().unwrap().domain_work(),
            resumed.latest_report().unwrap().domain_work()
        );
        assert_ne!(
            ordinary.incumbents()[0].occurrence_identity(),
            resumed.incumbents()[0].occurrence_identity()
        );
        assert_eq!(
            ordinary.incumbents()[0].state_identity(),
            resumed.incumbents()[0].state_identity()
        );
        assert_eq!(resumed.counters().yield_count(), 1);
        assert_eq!(resumed.counters().readmission_count(), 1);
        assert_eq!(resumed.counters().iteration_count(), 1);
        assert_eq!(resumed.counters().provider_work_unit_count(), 2);
        let ordinary_cleanup = ordinary
            .cleanup()
            .unwrap_or_else(|_| panic!("ordinary schedule must clean up"));
        let resumed_cleanup = resumed
            .cleanup()
            .unwrap_or_else(|_| panic!("readmitted schedule must clean up"));
        assert_eq!(ordinary_cleanup.counters().cleanup_attempt_count(), 1);
        assert_eq!(ordinary_cleanup.counters().cleanup_completion_count(), 1);
        assert_eq!(resumed_cleanup.counters().cleanup_attempt_count(), 1);
        assert_eq!(resumed_cleanup.counters().cleanup_completion_count(), 1);
    });
}

#[test]
fn workflow_yield_and_readmission_preserve_the_semantic_convergence_result() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let ordinary = workflow_converged_terminal(execution, resource_request);
        let WorkflowAdmissionFixture {
            runtime,
            operation,
            contract,
            managed,
            graph,
            bridge,
        } = workflow_admission_fixture(FixtureDisposition::YieldThenConverged, resource_request);
        let admitted =
            match runtime.admit_workflow_convergence_epoch(&operation, contract, managed, graph) {
                Ok(epoch) => epoch,
                Err(_) => panic!("yield-capable workflow authorities must admit"),
            };
        let epoch = match admitted.start() {
            Ok(epoch) => epoch,
            Err(_) => panic!("yield-capable workflow epoch must start"),
        };
        let started = match epoch.begin_stage_iteration(
            execution,
            WORKFLOW_STAGE,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "yielded-workflow-convergence-iteration",
            ),
        ) {
            Ok(started) => started,
            Err(_) => panic!("yield-capable workflow iteration must start"),
        };
        let paused = match started.advance(execution) {
            WorthQueryWorkflowConvergenceStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow provider must expose its installed safe point"),
        };
        let yielded = match paused.yield_iteration() {
            WorthQueryWorkflowConvergenceYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("eligible workflow iteration must preserve yielded authority"),
        };
        let resumed = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            WorthQueryWorkflowConvergenceReadmissionOutcome::Readmitted(readmitted) => {
                assert_committed_readmission_evidence(readmitted.readmission_evidence());
                readmitted.into_started()
            }
            _ => panic!("same-runtime workflow readmission must restore the provider"),
        };
        let outcome = match resumed.advance(execution) {
            WorthQueryWorkflowConvergenceStepOutcome::Completed(outcome) => outcome,
            _ => panic!("restored workflow provider must complete and rejoin its epoch"),
        };
        let resumed = match outcome {
            WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal) => terminal,
            _ => panic!("restored installed workflow comparator must converge"),
        };

        assert_eq!(ordinary.kind(), resumed.kind());
        assert_eq!(
            ordinary.latest_report().unwrap().decision(),
            resumed.latest_report().unwrap().decision()
        );
        assert_eq!(
            ordinary.latest_report().unwrap().domain_work(),
            resumed.latest_report().unwrap().domain_work()
        );
        assert_ne!(
            ordinary.incumbents()[0].occurrence_identity(),
            resumed.incumbents()[0].occurrence_identity()
        );
        assert_eq!(resumed.counters().yield_count(), 1);
        assert_eq!(resumed.counters().readmission_count(), 1);
        assert_eq!(resumed.counters().iteration_count(), 1);
        assert_eq!(resumed.counters().provider_work_unit_count(), 2);
        let ordinary_cleanup = ordinary.cleanup();
        let resumed_cleanup = resumed.cleanup();
        assert_eq!(ordinary_cleanup.counters().cleanup_attempt_count(), 1);
        assert_eq!(ordinary_cleanup.counters().cleanup_completion_count(), 1);
        assert_eq!(resumed_cleanup.counters().cleanup_attempt_count(), 1);
        assert_eq!(resumed_cleanup.counters().cleanup_completion_count(), 1);
        assert!(matches!(
            ordinary_cleanup,
            WorthQueryWorkflowConvergenceCleanupOutcome::Complete(_)
        ));
        assert!(matches!(
            resumed_cleanup,
            WorthQueryWorkflowConvergenceCleanupOutcome::Complete(_)
        ));
    });
}

#[test]
fn admitted_chunk_schedule_preserves_the_semantic_convergence_result() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let ordinary = workflow_converged_terminal(execution, resource_request);
        for width in [1, 8] {
            let chunked = chunked_converged_terminal(execution, width, resource_request);
            assert_eq!(ordinary.kind(), chunked.kind());
            assert_eq!(
                ordinary.latest_report().unwrap().decision(),
                chunked.latest_report().unwrap().decision()
            );
            assert_eq!(
                ordinary.latest_report().unwrap().domain_work(),
                chunked.latest_report().unwrap().domain_work()
            );
            assert_eq!(
                ordinary.incumbents()[0].state_identity(),
                chunked.incumbents()[0].state_identity()
            );
            assert_eq!(chunked.counters().provider_work_unit_count(), 1);
            assert!(matches!(
                chunked.cleanup(),
                WorthQueryWorkflowConvergenceCleanupOutcome::Complete(_)
            ));
        }
        assert!(matches!(
            ordinary.cleanup(),
            WorthQueryWorkflowConvergenceCleanupOutcome::Complete(_)
        ));
    });
}

fn chunked_converged_terminal(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    width: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryWorkflowConvergenceTerminal<WorthQueryConverged> {
    let epoch = workflow_epoch_fixture(
        FixtureDisposition::ChunkedConverged(width),
        resource_request,
    );
    let started = match epoch.begin_stage_iteration(
        execution,
        WORKFLOW_STAGE,
        WorthQueryManagedGraphCallRequest::new(
            WorthQueryGraphProviderCallKind::Project,
            format!("chunked-workflow-convergence-iteration-{width}"),
        ),
    ) {
        Ok(started) => started,
        Err(_) => panic!("chunked workflow iteration must start"),
    };
    let chunk = match started.advance(execution) {
        WorthQueryWorkflowConvergenceStepOutcome::ChunkReady(chunk) => chunk,
        _ => panic!("chunked workflow provider must expose its bounded projection"),
    };
    assert_eq!(
        chunk.queue_depth(),
        u64::try_from(width).expect("fixture chunk width must fit the queue counter")
    );
    assert_eq!(chunk.queue_capacity(), 8);
    let outcome = match chunk.acknowledge() {
        WorthQueryWorkflowConvergenceStepOutcome::Completed(outcome) => outcome,
        _ => panic!("acknowledged exact-capacity chunk must complete and rejoin"),
    };
    match outcome {
        WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal) => terminal,
        _ => panic!("chunk schedule must preserve convergence"),
    }
}

fn assert_committed_readmission_evidence(evidence: WorthQueryReadmissionEvidence) {
    assert_eq!(evidence.query_counters().committed_attempt_count(), 1);
    let bridge = evidence
        .bridge_counters()
        .expect("convergence readmission must carry Bridge owner evidence");
    assert_eq!(bridge.abort_count(), 0);
    assert_eq!(bridge.commit_count(), 1);
}
