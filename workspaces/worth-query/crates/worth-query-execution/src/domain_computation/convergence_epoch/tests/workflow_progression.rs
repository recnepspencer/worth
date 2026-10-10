use super::fixture::{workflow_epoch_fixture, FixtureDisposition, WORKFLOW_STAGE};
use crate::domain_computation::{
    WorthQueryConvergenceEpochDenialKind, WorthQueryConvergenceIterationStartFailureKind,
    WorthQueryConvergenceTerminalKind, WorthQueryGraphProviderCallKind,
    WorthQueryManagedGraphCallRequest, WorthQueryManagedStepContractDenialKind,
    WorthQueryStartedWorkflowConvergenceIteration, WorthQueryWorkflowConvergenceCleanupOutcome,
    WorthQueryWorkflowConvergenceIterationOutcome, WorthQueryWorkflowConvergenceStepOutcome,
    WorthQueryWorkflowGraphExecutionStartFailureKind,
};

#[test]
fn real_installed_workflow_seals_its_evidence_stage_and_converges() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let epoch = workflow_epoch_fixture(FixtureDisposition::Converged, resource_request);
        let rejection = match epoch.begin_stage_iteration(
            execution,
            "foreign-stage",
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "wrong-workflow-stage",
            ),
        ) {
            Ok(_) => panic!("a stage not sealed by convergence admission entered iteration"),
            Err(rejection) => rejection,
        };
        assert_eq!(
            rejection.denial().kind(),
            WorthQueryConvergenceEpochDenialKind::WorkflowEvidenceStageMismatch
        );
        let epoch = rejection.into_epoch();
        let started = match epoch.begin_stage_iteration(
            execution,
            WORKFLOW_STAGE,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "workflow-convergence-iteration",
            ),
        ) {
            Ok(started) => started,
            Err(_) => panic!("the sealed workflow evidence stage must start iteration"),
        };
        let outcome = match started.advance(execution) {
            WorthQueryWorkflowConvergenceStepOutcome::Completed(outcome) => outcome,
            _ => panic!("single-step workflow fixture provider must complete and rejoin its epoch"),
        };
        let terminal = match outcome {
            WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal) => terminal,
            _ => panic!("installed workflow convergence decision must remain distinct"),
        };
        assert_eq!(
            terminal.kind(),
            WorthQueryConvergenceTerminalKind::Converged
        );
        assert_eq!(terminal.incumbents().len(), 1);
        let incumbent = &terminal.incumbents()[0];
        let report = terminal
            .latest_report()
            .expect("workflow comparison must retain its report");
        assert_eq!(
            incumbent.report_evidence_identity(),
            report.evidence_identity()
        );
        assert_eq!(
            incumbent.state_identity(),
            report.decision().state_identity()
        );
        assert_ne!(incumbent.occurrence_identity(), "candidate-1");
        assert_eq!(report.decision().candidate_selection_key(), "candidate-1");
        let cleanup = terminal.cleanup();
        assert_eq!(cleanup.counters().iteration_count(), 1);
        assert_eq!(cleanup.counters().provider_work_unit_count(), 1);
        assert!(matches!(
            cleanup,
            WorthQueryWorkflowConvergenceCleanupOutcome::Complete(_)
        ));
    });
}

#[test]
fn same_semantic_candidate_in_same_stage_workflow_peers_has_distinct_occurrences() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let first = start_workflow_peer(execution, "workflow-peer", resource_request);
        let second = start_workflow_peer(execution, "workflow-peer", resource_request);
        let first = complete_workflow_peer(execution, first);
        let second = complete_workflow_peer(execution, second);

        assert_ne!(first, second);
    });
}

fn start_workflow_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    scope: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryStartedWorkflowConvergenceIteration {
    let epoch = workflow_epoch_fixture(FixtureDisposition::Converged, resource_request);
    epoch
        .begin_stage_iteration(
            execution,
            WORKFLOW_STAGE,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, scope),
        )
        .unwrap_or_else(|_| panic!("real workflow peer must begin"))
}

fn complete_workflow_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    started: WorthQueryStartedWorkflowConvergenceIteration,
) -> String {
    let outcome = match started.advance(execution) {
        WorthQueryWorkflowConvergenceStepOutcome::Completed(outcome) => outcome,
        _ => panic!("real workflow peer must complete"),
    };
    let terminal = match outcome {
        WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal) => terminal,
        _ => panic!("real workflow peer must converge"),
    };
    let incumbent = &terminal.incumbents()[0];
    assert_eq!(
        incumbent.report_evidence_identity(),
        terminal.latest_report().unwrap().evidence_identity()
    );
    assert_eq!(
        terminal
            .latest_report()
            .unwrap()
            .decision()
            .candidate_selection_key(),
        "candidate-1"
    );
    incumbent.occurrence_identity().to_owned()
}

#[test]
fn incompatible_stage_queue_contract_denies_before_iteration_and_terminates_cleanly() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let epoch = workflow_epoch_fixture(
            FixtureDisposition::StageQueueContractMismatch,
            resource_request,
        );
        let rejection = match epoch.begin_stage_iteration(
            execution,
            WORKFLOW_STAGE,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Project,
                "stage-queue-contract-mismatch",
            ),
        ) {
            Ok(_) => {
                panic!("a stage queue contract wider than Signal entered convergence iteration")
            }
            Err(rejection) => rejection,
        };
        assert_eq!(rejection.denial().counters().iteration_count(), 0);

        let (denial, outcome) = rejection.terminate().into_parts();
        assert_eq!(
            denial.kind(),
            WorthQueryConvergenceEpochDenialKind::ManagedIterationStart(
                WorthQueryConvergenceIterationStartFailureKind::Workflow(
                    WorthQueryWorkflowGraphExecutionStartFailureKind::StepContract(
                        WorthQueryManagedStepContractDenialKind::QueueDepthExceeded,
                    ),
                ),
            )
        );
        let terminal = match outcome {
            WorthQueryWorkflowConvergenceIterationOutcome::Indeterminate(terminal) => terminal,
            _ => panic!("an irrecoverable managed start denial must fail closed as indeterminate"),
        };
        assert_eq!(
            terminal.kind(),
            WorthQueryConvergenceTerminalKind::Indeterminate
        );
        assert_eq!(terminal.counters().iteration_count(), 0);
        assert_eq!(terminal.counters().provider_work_unit_count(), 0);
        let cleanup = terminal.cleanup();
        assert_eq!(cleanup.counters().cleanup_attempt_count(), 1);
        assert_eq!(cleanup.counters().cleanup_completion_count(), 1);
        assert!(matches!(
            cleanup,
            WorthQueryWorkflowConvergenceCleanupOutcome::Complete(_)
        ));
    });
}
