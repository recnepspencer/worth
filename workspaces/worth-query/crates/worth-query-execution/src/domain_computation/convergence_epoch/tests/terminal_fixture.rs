use super::fixture::{
    direct_epoch_fixture, workflow_epoch_fixture, FixtureDisposition, WORKFLOW_STAGE,
};
use crate::domain_computation::{
    WorthQueryConverged, WorthQueryDirectConvergenceIterationOutcome,
    WorthQueryDirectConvergenceStepOutcome, WorthQueryDirectConvergenceTerminal,
    WorthQueryGraphProviderCallKind, WorthQueryIndeterminate, WorthQueryManagedGraphCallRequest,
    WorthQueryStableWithoutProof, WorthQueryWorkflowConvergenceIterationOutcome,
    WorthQueryWorkflowConvergenceStepOutcome, WorthQueryWorkflowConvergenceTerminal,
};

pub(super) fn direct_terminal_outcome(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    disposition: FixtureDisposition,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDirectConvergenceIterationOutcome {
    let epoch = direct_epoch_fixture(disposition, resource_request);
    let started = match epoch.begin_iteration(
        execution,
        WorthQueryManagedGraphCallRequest::new(
            WorthQueryGraphProviderCallKind::Observe,
            "terminal-matrix",
        ),
    ) {
        Ok(started) => started,
        Err(_) => panic!("terminal matrix iteration must start"),
    };
    match started.advance(execution) {
        WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
        _ => panic!("terminal matrix provider must complete and rejoin"),
    }
}

pub(super) fn converged_terminal(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDirectConvergenceTerminal<WorthQueryConverged> {
    match direct_terminal_outcome(execution, FixtureDisposition::Converged, resource_request) {
        WorthQueryDirectConvergenceIterationOutcome::Converged(terminal) => terminal,
        _ => panic!("converged fixture reached the wrong terminal"),
    }
}

pub(super) fn stable_without_proof_terminal(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDirectConvergenceTerminal<WorthQueryStableWithoutProof> {
    match direct_terminal_outcome(
        execution,
        FixtureDisposition::StableWithoutProof,
        resource_request,
    ) {
        WorthQueryDirectConvergenceIterationOutcome::StableWithoutProof(terminal) => terminal,
        _ => panic!("stable-without-proof fixture reached the wrong terminal"),
    }
}

pub(super) fn indeterminate_terminal(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    disposition: FixtureDisposition,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDirectConvergenceTerminal<WorthQueryIndeterminate> {
    match direct_terminal_outcome(execution, disposition, resource_request) {
        WorthQueryDirectConvergenceIterationOutcome::Indeterminate(terminal) => terminal,
        _ => panic!("indeterminate fixture reached the wrong terminal"),
    }
}

pub(super) fn workflow_converged_terminal(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryWorkflowConvergenceTerminal<WorthQueryConverged> {
    match workflow_terminal_outcome(execution, FixtureDisposition::Converged, resource_request) {
        WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal) => terminal,
        _ => panic!("ordinary workflow comparator must converge"),
    }
}

pub(super) fn workflow_indeterminate_terminal(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    disposition: FixtureDisposition,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryWorkflowConvergenceTerminal<WorthQueryIndeterminate> {
    match workflow_terminal_outcome(execution, disposition, resource_request) {
        WorthQueryWorkflowConvergenceIterationOutcome::Indeterminate(terminal) => terminal,
        _ => panic!("ordinary workflow comparator must remain indeterminate"),
    }
}

fn workflow_terminal_outcome(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    disposition: FixtureDisposition,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryWorkflowConvergenceIterationOutcome {
    let epoch = workflow_epoch_fixture(disposition, resource_request);
    let started = match epoch.begin_stage_iteration(
        execution,
        WORKFLOW_STAGE,
        WorthQueryManagedGraphCallRequest::new(
            WorthQueryGraphProviderCallKind::Observe,
            "ordinary-workflow-convergence-iteration",
        ),
    ) {
        Ok(started) => started,
        Err(_) => panic!("ordinary workflow iteration must start"),
    };
    match started.advance(execution) {
        WorthQueryWorkflowConvergenceStepOutcome::Completed(outcome) => outcome,
        _ => panic!("ordinary workflow provider must complete and rejoin"),
    }
}
