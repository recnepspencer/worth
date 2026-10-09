//! Failed workflow stage preserves governed work and requires recovery.

use super::*;

#[test]
fn failed_workflow_stage_preserves_governed_work_and_requires_recovery() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let provider_anchor = provider_anchor(WorkflowStageBehavior::Fail);
        let provider_support = provider_anchor.resource_support().clone();
        let graph = installed_graph(&installer, "uncertain-workflow-graph", provider_anchor);
        let runtime = installed_runtime(installer, "uncertain workflow");
        let operation_resources = admitted_plan("uncertain-workflow", 8);
        let stage_resources = admitted_plan_with_graph_support(
            "uncertain-workflow:stage",
            4,
            graph.role(),
            provider_support,
        );
        let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
            operation_resources,
            BTreeMap::from([("stage".to_owned(), stage_resources)]),
        );
        let operation = workflow_authority_with_stage_graph(
            &runtime,
            &resources,
            "stage",
            &graph,
            WorthQueryOperationGraphAccess::Project,
        );
        let running = admitted_workflow(&runtime, &operation, resources);
        let active = running
            .begin_stage_graph_execution(
                execution,
                "stage",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Project,
                    "uncertain-workflow-stage",
                ),
            )
            .expect("installed stage resources should start the provider call");
        let terminal = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Failed(terminal) => terminal,
            _ => panic!("failed workflow provider advanced the managed lane"),
        };
        assert_eq!(
            terminal.provider_work().session_disposition(),
            WorthQueryManagedProviderSessionDisposition::Uncertain
        );
        assert_eq!(terminal.provider_work().abandoned_call_count(), 1);
        assert_eq!(terminal.provider_work().completed_work_units(), 3);
        assert_eq!(
            workflow_cleanup(terminal.cleanup())
                .inspection()
                .disposition(),
            WorthQueryManagedRunCleanupDisposition::RecoveryRequired
        );
    });
}
