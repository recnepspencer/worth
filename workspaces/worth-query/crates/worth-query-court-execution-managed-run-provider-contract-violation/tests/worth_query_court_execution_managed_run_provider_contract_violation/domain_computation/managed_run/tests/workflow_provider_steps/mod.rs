pub use std::sync::Arc;
pub use std::sync::atomic::AtomicUsize;
pub use std::collections::BTreeMap;
pub use std::sync::atomic::Ordering;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::WorkflowStageBehavior;
pub use worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan;
pub use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
pub use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;
pub use worth_query_execution::facade::domain_computation::WorthQueryManagedProviderSessionDisposition;
pub use worth_query_execution::facade::domain_computation::WorthQueryManagedRunCleanupDisposition;
pub use worth_query_installation::facade::WorthQueryOperationGraphAccess;
pub use worth_query_execution::facade::domain_computation::WorthQueryWorkflowGraphStepOutcome;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::admitted_plan;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::admitted_plan_with_graph_support;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::admitted_workflow;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::assert_workflow_artifact_evidence;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_graph;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_runtime;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::provider_anchor;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provider_work;
pub use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::workflow_authority_with_stage_graph;
pub use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::workflow_authority_with_stage_graph_and_output_artifact;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::workflow_cleanup;

pub use crate::process::domain_computation::managed_run::tests::*;
#[test]
fn workflow_stage_provider_call_uses_stage_resources_and_receipt_evidence() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let provider_anchor = provider_anchor(WorkflowStageBehavior::Project);
        let provider_support = provider_anchor.resource_support().clone();
        let graph = installed_graph(&installer, "workflow-graph", provider_anchor);
        let runtime = installed_runtime(installer, "workflow graph");
        let operation_resources = admitted_plan("workflow-graph-binding", 8);
        let stage_resources = admitted_plan_with_graph_support(
            "workflow-graph-binding:stage",
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
        let running = admitted_workflow(&runtime, &operation, resources, resource_request);
        let installed_running_phase = execution.phase_for_workflow(&running);

        let active = running
            .begin_stage_graph_execution(
                &installed_running_phase,
                "stage",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Project,
                    "workflow-stage-project",
                ),
            )
            .expect("installed stage resources should start the bounded provider");
        let pending = match active.advance(&installed_running_phase) {
            WorthQueryWorkflowGraphStepOutcome::ChunkReady(pending) => pending,
            _ => panic!("workflow stage did not expose its bounded result chunk"),
        };
        assert_eq!(pending.queue_depth(), 1);
        let completion = match pending.acknowledge() {
            WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("acknowledged workflow stage did not complete"),
        };
        let stream = completion
            .receipt()
            .graph_read_stream_evidence()
            .expect("workflow managed projection should be streamed");
        assert_eq!(stream.row_count(), 1);
        assert_eq!(stream.product().rows().count(), 1);
        assert_eq!(
            completion.receipt().work_report().output_retained_bytes(),
            stream.retained_bytes()
        );
        let terminal = completion
            .into_running()
            .completed()
            .expect("step-bound stage work should complete");
        assert_eq!(terminal.provider_work().completed_work_units(), 3);
        let cleanup = workflow_cleanup(terminal.cleanup());
        assert_eq!(
            cleanup
                .inspection()
                .provider_work()
                .admitted_receipt_count(),
            1
        );
    });
}
#[test]
fn workflow_step_derives_artifact_and_checkpoint_evidence_from_governed_ports() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let disposed = Arc::new(AtomicUsize::new(0));
        let provider_anchor = provider_anchor(WorkflowStageBehavior::ArtifactCheckpoint(
            Arc::clone(&disposed),
        ));
        let provider_support = provider_anchor.resource_support().clone();
        let graph = installed_graph(&installer, "workflow-artifact-graph", provider_anchor);
        let runtime = installed_runtime(installer, "workflow artifact");
        let operation_resources = admitted_plan("workflow-artifact-step", 8);
        let stage_resources = admitted_plan_with_graph_support(
            "workflow-artifact-step:producer",
            4,
            graph.role(),
            provider_support,
        );
        let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
            operation_resources,
            BTreeMap::from([("producer".to_owned(), stage_resources)]),
        );
        let output =
            worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::installed_artifact_contract_for_managed_run(
            );
        let operation = workflow_authority_with_stage_graph_and_output_artifact(
            &runtime,
            &resources,
            "producer",
            &graph,
            WorthQueryOperationGraphAccess::Observe,
            output,
        );
        let running = admitted_workflow(&runtime, &operation, resources, resource_request);
        let installed_running_phase = execution.phase_for_workflow(&running);

        let active = running
            .begin_stage_graph_execution(
                &installed_running_phase,
                "producer",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "workflow-artifact-step",
                ),
            )
            .expect("workflow artifact provider should start");
        let completion = match active.advance(&installed_running_phase) {
            WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("governed artifact and checkpoint step did not complete"),
        };
        let report = completion.receipt().work_report();
        assert_eq!(report.produced_artifact_count(), 1);
        assert_eq!(report.retained_artifact_count(), 1);
        assert_eq!(report.disposed_artifact_count(), 0);
        assert_eq!(report.retained_bytes(), 1);
        assert_eq!(disposed.load(Ordering::Acquire), 1);
        let terminal = completion
            .into_running()
            .completed()
            .expect("governed artifact provider work should settle");
        assert!(terminal.provider_work().checkpoint_available());
        assert_eq!(terminal.provider_work().produced_artifact_count(), 1);
        assert_eq!(terminal.provider_work().retained_artifact_count(), 0);
        assert_eq!(terminal.provider_work().disposed_artifact_count(), 1);
        assert_eq!(terminal.provider_work().retained_bytes(), 0);
        assert_workflow_artifact_evidence(terminal.artifact_evidence(), (1, 0, 1, 0));
        let cleanup = workflow_cleanup(terminal.cleanup());
        assert_eq!(
            cleanup.inspection().disposition(),
            WorthQueryManagedRunCleanupDisposition::CleanupComplete
        );
    });
}

#[path = "failed_workflow_stage_preserves_governed_work_and_requires_recovery/mod.rs"]
pub mod failed_workflow_stage_preserves_governed_work_and_requires_recovery;
