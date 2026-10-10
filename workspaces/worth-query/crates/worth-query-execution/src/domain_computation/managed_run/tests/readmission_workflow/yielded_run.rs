use super::*;

pub(super) fn yielded_workflow_fixture(
    execution: &crate::domain_computation::primary_graph::TestHostRequest<'_>,

    provider: YieldProvider,
    stage_identity: &str,
    retain_artifact: bool,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    Arc<crate::domain_computation::WorthQueryArtifactProductionAuthority>,
    Option<crate::domain_computation::WorthQueryMoveOnlyArtifactHandle>,
) {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            provider,
        ),
    );
    let provider_support = provider_anchor.resource_support().clone();
    let graph = super::workflow_provider_steps::installed_graph(
        &installer,
        "workflow-readmission-graph",
        provider_anchor,
    );
    let runtime =
        super::workflow_provider_steps::installed_runtime(installer, "workflow readmission");
    let operation_resources =
        crate::domain_computation::provider_session::admitted_yield_plan("workflow-readmission", 8);
    let stage_resource_identity = format!("workflow-readmission:{stage_identity}");
    let stage_resources = admitted_plan_with_graph_support(
        &stage_resource_identity,
        8,
        graph.role(),
        provider_support,
    );
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([(stage_identity.to_owned(), stage_resources)]),
    );
    let output =
        crate::domain_computation::artifact_owner::installed_artifact_contract_for_managed_run();
    let operation = workflow_authority_with_stage_graph_and_output_artifact(
        &runtime,
        &resources,
        stage_identity,
        &graph,
        WorthQueryOperationGraphAccess::Observe,
        output,
    );
    let attempt = runtime
        .start_workflow_resource_attempt(&operation, resources)
        .expect("workflow resources should reserve");
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&operation, attempt, lower.read_request(), resource_request)
        .expect("workflow should admit")
        .start()
        .expect("workflow should start");
    let old_producer = running
        .artifacts()
        .production_authority(stage_identity)
        .expect("producer stage should validate")
        .expect("producer stage should own output authority");
    let retained_artifact = retain_artifact.then(|| {
        let admission = crate::domain_computation::WorthQueryArtifactProductionAuthority::admit(
            &old_producer,
            WorthQueryArtifactProductionEvidence::new("readmission-cleanup", "retained-artifact"),
        );
        crate::domain_computation::WorthQueryArtifactProductionAuthority::register_exact(
            &old_producer,
            admission,
            ReadmissionArtifact,
        )
        .expect("retained workflow artifact must register before production freezes")
    });
    let active = {
        let installed_fixture_phase = running.fixture_phase(execution);
        running.begin_stage_graph_execution(
            &installed_fixture_phase,
            stage_identity,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "workflow-readmission",
            ),
        )
    }
    .expect("workflow provider should begin");
    let paused = {
        let fixture_phase_result = {
            let installed_fixture_phase = active.fixture_phase(execution);
            active.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow provider did not pause"),
        }
    };
    let yielded = match paused.yield_run() {
        crate::domain_computation::WorthQueryWorkflowYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("eligible workflow did not yield"),
    };
    (
        yielded,
        lower.bridge,
        runtime,
        old_producer,
        retained_artifact,
    )
}
