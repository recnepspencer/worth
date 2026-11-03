use std::collections::BTreeMap;
use std::sync::Arc;

use crate::domain_computation::managed_run::tests::*;
use worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionPlanView;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionLifecycle;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionToken;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionTokenAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionView;
use worth_query_execution::facade::domain_computation::WorthQuerySessionAbortOutcome;

pub struct WorkflowSessionProvider;
pub struct WorkflowSessionExecution;

impl WorthQueryGraphProviderExecution for WorkflowSessionExecution {
    fn advance(
        &mut self,
        _step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        unreachable!("workflow session protocol does not use the one-shot callback")
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for WorkflowSessionProvider {
    type Execution = WorkflowSessionExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::execution_resource_support(
            "workflow-session-protocol",
            8,
        )
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        _start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        unreachable!("workflow session protocol must not call the legacy provider path")
    }
}

impl WorthQueryProviderSessionLifecycle for WorkflowSessionProvider {
    fn readmit_provider_plan(
        &self,
        plan: &WorthQueryProviderExecutionPlanView<'_>,
        admission: WorthQueryProviderSessionTokenAdmission,
    ) -> Result<WorthQueryProviderSessionToken, WorthQueryProviderSessionFailure> {
        assert_eq!(plan.contract().scope().stage_identity(), Some("stage"));
        admission.admit("workflow-physical-session")
    }

    fn prepare_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        Ok(())
    }

    fn prepare_staged_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        Ok(())
    }

    fn commit_prepared_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
        _allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription,
        worth_query_execution::facade::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        Ok(
            worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription::new(
                "workflow commit",
            )
            .expect("fixture description is valid"),
        )
    }

    fn abort_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        Ok(
            worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription::new("workflow abort")
                .expect("fixture description is valid"),
        )
    }
}

#[test]
fn workflow_stage_uses_stage_resources_and_scope_in_the_same_protocol() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (mut running, graph) = workflow_session_run("workflow-session", resource_request);
        let installed_running_phase = execution.phase_for_workflow(&running);

        {
            let plan = running
                .admit_stage_provider_execution_plan("stage", &graph)
                .expect("installed workflow stage should admit its provider plan");
            assert_eq!(plan.contract().scope().stage_identity(), Some("stage"));
            assert_eq!(plan.contract().read_closure(), ["managed-graph:project"]);
            let outcome = plan
                .readmit(&installed_running_phase)
                .expect("workflow plan should readmit")
                .prepare()
                .expect("workflow session should prepare")
                .bind_reads_and_effects()
                .abort();
            assert!(matches!(outcome, WorthQuerySessionAbortOutcome::Aborted(_)));
        }
        cleanup_workflow(running);
    });
}

#[test]
fn workflow_plan_carries_the_exact_installed_artifact_contract() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let output =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::installed_artifact_contract_for_managed_run();
    let expected = format!(
        "stage-output|admission={}|contract={}",
        output.admission_identity().render_support_hex(),
        output.contract().identity().as_str(),
    );
    let (mut running, graph) = workflow_session_run_with_output(
        "workflow-artifact-session",
        Some(output),
        resource_request,
    );
    let plan = running
        .admit_stage_provider_execution_plan("stage", &graph)
        .expect("stage with an exact installed output artifact should admit");
    assert_eq!(plan.contract().artifact_closure(), [expected]);
    drop(plan);
    cleanup_workflow(running);
}

#[test]
fn foreign_graph_and_wrong_stage_deny_before_provider_readmission() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (mut first, first_graph) = workflow_session_run("first-session", resource_request);
    let (mut second, second_graph) = workflow_session_run("second-session", resource_request);
    let foreign = first
        .admit_stage_provider_execution_plan("stage", &second_graph)
        .expect_err("foreign installed graph authority must not substitute");
    assert_eq!(
        foreign.kind(),
        WorthQueryProviderSessionDenialKind::ForeignGraphAuthority
    );
    let wrong_stage = second
        .admit_stage_provider_execution_plan("other-stage", &first_graph)
        .expect_err("undeclared workflow stage must not admit");
    assert_eq!(
        wrong_stage.kind(),
        WorthQueryProviderSessionDenialKind::UndeclaredOperationScope
    );
    cleanup_workflow(first);
    cleanup_workflow(second);
}

#[test]
fn protocol_work_is_constant_while_closure_copy_tracks_only_declared_width() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (mut direct, direct_graph) = managed_session_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        WorkflowSessionProvider,
        false,
        resource_request,
    );
    let direct_counters = direct
        .admit_provider_execution_plan(&direct_graph)
        .expect("direct read plan should admit")
        .counters();
    let (mut effect, effect_graph) = managed_session_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        WorkflowSessionProvider,
        true,
        resource_request,
    );
    let effect_counters = effect
        .admit_provider_execution_plan(&effect_graph)
        .expect("direct effect plan should admit")
        .counters();
    assert_eq!(direct_counters.authority_checks(), 1);
    assert_eq!(effect_counters.authority_checks(), 1);
    assert_eq!(direct_counters.provider_calls(), 0);
    assert_eq!(effect_counters.provider_calls(), 0);
    assert_eq!(direct_counters.closure_items_bound(), 1);
    assert_eq!(effect_counters.closure_items_bound(), 2);
    cleanup_direct(direct);
    cleanup_direct(effect);
}

pub fn workflow_session_run(
    label: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun,
    WorthQueryInstalledGraphParticipationAuthority,
) {
    workflow_session_run_with_output(label, None, resource_request)
}

pub fn workflow_session_run_with_output(
    label: &str,
    output: Option<
        Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
    >,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun,
    WorthQueryInstalledGraphParticipationAuthority,
) {
    let (installer, graph, provider_support) = install_workflow_session_graph();
    let resources = workflow_session_resources(label, &graph, provider_support);
    let runtime =
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_runtime(installer, "workflow session protocol");
    let operation = match output {
        Some(output) => workflow_authority_with_stage_graph_and_output_artifact(
            &runtime,
            &resources,
            "stage",
            &graph,
            WorthQueryOperationGraphAccess::Project,
            output,
        ),
        None => workflow_authority_with_stage_graph(
            &runtime,
            &resources,
            "stage",
            &graph,
            WorthQueryOperationGraphAccess::Project,
        ),
    };
    let running = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::admitted_workflow(
        &runtime,
        &operation,
        resources,
        resource_request,
    );
    (running, graph)
}

pub fn install_workflow_session_graph() -> (
    WorthQueryExecutionRuntimeInstaller,
    WorthQueryInstalledGraphParticipationAuthority,
    worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
) {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install_session_capable::<ManagedGraph, _>(
            WorkflowSessionProvider,
        ),
    );
    let provider_identity = anchor.provider_identity();
    let provider_support = anchor.resource_support().clone();
    let graph = WorthQueryInstalledGraphParticipationAuthority::install(
        installer.installation_runtime(),
        "managed-graph",
        provider_identity,
        false,
        Option::<String>::None,
        anchor,
    )
    .expect("workflow session graph should install");
    (installer, graph, provider_support)
}

pub fn workflow_session_resources(
    label: &str,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    provider_support: worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
) -> WorthQueryAdmittedWorkflowResourcePlan {
    let binding = format!("{label}-binding");
    WorthQueryAdmittedWorkflowResourcePlan::assemble(
        admitted_plan(&binding, 8),
        BTreeMap::from([(
            "stage".to_owned(),
            admitted_plan_with_graph_support(
                &format!("{binding}:stage"),
                8,
                graph.role(),
                provider_support,
            ),
        )]),
    )
}

pub fn cleanup_workflow(
    running: worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun,
) {
    let outcome = running
        .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed)
        .cleanup();
    assert!(matches!(
        outcome,
        WorthQueryWorkflowRunCleanupOutcome::Complete(_)
    ));
}

pub fn cleanup_direct(running: WorthQueryRunningDirectRun) {
    running
        .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed)
        .cleanup()
        .expect("direct protocol cost fixture cleanup should complete");
}
