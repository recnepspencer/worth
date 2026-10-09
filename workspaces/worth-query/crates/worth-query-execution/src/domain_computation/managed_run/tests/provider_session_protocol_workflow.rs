use std::collections::BTreeMap;
use std::sync::Arc;

use super::*;
use crate::domain_computation::{
    WorthQueryProviderExecutionPlanView, WorthQueryProviderSessionDenialKind,
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionLifecycle,
    WorthQueryProviderSessionToken, WorthQueryProviderSessionTokenAdmission,
    WorthQueryProviderSessionView, WorthQuerySessionCommitOrAbortOutcome,
};

struct WorkflowSessionProvider;
struct WorkflowSessionExecution;

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
        crate::domain_computation::provider_session::execution_resource_support(
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
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        Ok(
            crate::domain_computation::WorthQueryProviderTerminalDescription::new(
                "workflow commit",
            )
            .expect("fixture description is valid"),
        )
    }

    fn abort_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        Ok(
            crate::domain_computation::WorthQueryProviderTerminalDescription::new("workflow abort")
                .expect("fixture description is valid"),
        )
    }
}

#[test]
fn workflow_stage_uses_stage_resources_and_scope_in_the_same_protocol() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (mut running, graph) = workflow_session_run("workflow-session");
        {
            let plan = running
                .admit_stage_provider_execution_plan("stage", &graph)
                .expect("installed workflow stage should admit its provider plan");
            assert_eq!(plan.contract().scope().stage_identity(), Some("stage"));
            assert_eq!(plan.contract().read_closure(), ["managed-graph:project"]);
            let outcome = plan
                .readmit(execution)
                .expect("workflow plan should readmit")
                .prepare()
                .expect("workflow session should prepare")
                .bind_reads_and_effects()
                .abort();
            assert!(matches!(
                outcome,
                WorthQuerySessionCommitOrAbortOutcome::Aborted(_)
            ));
        }
        cleanup_workflow(running);
    });
}

#[test]
fn workflow_plan_carries_the_exact_installed_artifact_contract() {
    let output =
        crate::domain_computation::artifact_owner::installed_artifact_contract_for_managed_run();
    let expected = format!(
        "stage-output|admission={}|contract={}",
        output.admission_identity().render_support_hex(),
        output.contract().identity().as_str(),
    );
    let (mut running, graph) =
        workflow_session_run_with_output("workflow-artifact-session", Some(output));
    let plan = running
        .admit_stage_provider_execution_plan("stage", &graph)
        .expect("stage with an exact installed output artifact should admit");
    assert_eq!(plan.contract().artifact_closure(), [expected]);
    drop(plan);
    cleanup_workflow(running);
}

#[test]
fn foreign_graph_and_wrong_stage_deny_before_provider_readmission() {
    let (mut first, first_graph) = workflow_session_run("first-session");
    let (mut second, second_graph) = workflow_session_run("second-session");
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
    let (mut direct, direct_graph) = managed_session_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        WorkflowSessionProvider,
        false,
    );
    let direct_counters = direct
        .admit_provider_execution_plan(&direct_graph)
        .expect("direct read plan should admit")
        .counters();
    let (mut effect, effect_graph) = managed_session_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        WorkflowSessionProvider,
        true,
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

fn workflow_session_run(
    label: &str,
) -> (
    crate::domain_computation::WorthQueryRunningWorkflowRun,
    WorthQueryInstalledGraphParticipationAuthority,
) {
    workflow_session_run_with_output(label, None)
}

fn workflow_session_run_with_output(
    label: &str,
    output: Option<
        Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
    >,
) -> (
    crate::domain_computation::WorthQueryRunningWorkflowRun,
    WorthQueryInstalledGraphParticipationAuthority,
) {
    let (installer, graph, provider_support) = install_workflow_session_graph();
    let resources = workflow_session_resources(label, &graph, provider_support);
    let runtime =
        super::workflow_provider_steps::installed_runtime(installer, "workflow session protocol");
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
    let running =
        super::workflow_provider_steps::admitted_workflow(&runtime, &operation, resources);
    (running, graph)
}

fn install_workflow_session_graph() -> (
    WorthQueryExecutionRuntimeInstaller,
    WorthQueryInstalledGraphParticipationAuthority,
    worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
) {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install_session_capable::<ManagedGraph, _>(
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

fn workflow_session_resources(
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

fn cleanup_workflow(running: crate::domain_computation::WorthQueryRunningWorkflowRun) {
    let outcome = running
        .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed)
        .cleanup();
    assert!(matches!(
        outcome,
        WorthQueryWorkflowRunCleanupOutcome::Complete(_)
    ));
}

fn cleanup_direct(running: WorthQueryRunningDirectRun) {
    running
        .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed)
        .cleanup()
        .expect("direct protocol cost fixture cleanup should complete");
}
