use super::fixture::{
    workflow_admission_fixture, FixtureDisposition, WorkflowAdmissionFixture, WORKFLOW_STAGE,
};
use crate::domain_computation::{
    WorthQueryAdmittedWorkflowRun, WorthQueryConvergenceEpochDenialKind,
    WorthQueryExecutionBoundOperationAuthority, WorthQueryExecutionRuntime,
    WorthQueryGraphProviderCallKind, WorthQueryManagedGraphCallRequest,
    WorthQueryWorkflowConvergenceIterationOutcome, WorthQueryWorkflowConvergenceStepOutcome,
};
use worth_query_admission::facade::convergence_epoch::WorthQueryAdmittedConvergenceContract;
use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;

#[test]
fn foreign_workflow_run_is_denied_and_each_rightful_world_still_completes() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let WorkflowAdmissionFixture {
            runtime: first_runtime,
            operation: first_operation,
            contract: first_contract,
            managed: first_managed,
            graph: first_graph,
            bridge: _,
        } = workflow_admission_fixture(FixtureDisposition::Converged);
        let WorkflowAdmissionFixture {
            runtime: second_runtime,
            operation: second_operation,
            contract: second_contract,
            managed: second_managed,
            graph: second_graph,
            bridge: _,
        } = workflow_admission_fixture(FixtureDisposition::Converged);
        let expected_contract = first_contract.identity().to_owned();
        let expected_managed = second_managed.identity().to_owned();
        let expected_graph = first_graph.authority_identity().to_owned();

        let rejection = match first_runtime.admit_workflow_convergence_epoch(
            &first_operation,
            first_contract,
            second_managed,
            first_graph,
        ) {
            Ok(_) => panic!("foreign workflow managed run entered convergence"),
            Err(rejection) => rejection,
        };
        assert_eq!(
            rejection.denial().kind(),
            WorthQueryConvergenceEpochDenialKind::ManagedRunOperationMismatch
        );
        let counters = rejection.denial().counters();
        assert_eq!(counters.operation_authority_check_count(), 1);
        assert_eq!(counters.contract_authority_check_count(), 1);
        assert_eq!(counters.managed_run_authority_check_count(), 1);
        assert_eq!(counters.graph_authority_check_count(), 0);

        let (first_contract, second_managed, first_graph) = rejection.into_parts();
        assert_eq!(first_contract.identity(), expected_contract);
        assert_eq!(second_managed.identity(), expected_managed);
        assert_eq!(first_graph.authority_identity(), expected_graph);

        admit_and_complete(
            execution,
            first_runtime,
            first_operation,
            first_contract,
            first_managed,
            first_graph,
        );
        admit_and_complete(
            execution,
            second_runtime,
            second_operation,
            second_contract,
            second_managed,
            second_graph,
        );
    });
}

fn admit_and_complete(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    runtime: WorthQueryExecutionRuntime,
    operation: WorthQueryExecutionBoundOperationAuthority,
    contract: WorthQueryAdmittedConvergenceContract,
    managed: WorthQueryAdmittedWorkflowRun,
    graph: WorthQueryInstalledGraphParticipationAuthority,
) {
    let admitted =
        match runtime.admit_workflow_convergence_epoch(&operation, contract, managed, graph) {
            Ok(admitted) => admitted,
            Err(_) => panic!("each recovered authority set must admit only in its rightful world"),
        };
    let epoch = match admitted.start() {
        Ok(epoch) => epoch,
        Err(_) => panic!("rightful workflow authority must start"),
    };
    let started = match epoch.begin_stage_iteration(
        execution,
        WORKFLOW_STAGE,
        WorthQueryManagedGraphCallRequest::new(
            WorthQueryGraphProviderCallKind::Observe,
            "workflow-foreign-world-recovery",
        ),
    ) {
        Ok(started) => started,
        Err(_) => panic!("rightful workflow stage must begin"),
    };
    let outcome = match started.advance(execution) {
        WorthQueryWorkflowConvergenceStepOutcome::Completed(outcome) => outcome,
        _ => panic!("rightful workflow provider must complete"),
    };
    assert!(matches!(
        outcome,
        WorthQueryWorkflowConvergenceIterationOutcome::Converged(_)
    ));
}
