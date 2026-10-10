use super::*;

struct WorkflowAbandonProvider;

struct WorkflowAbandonExecution {
    step_ordinal: u8,
}

impl WorthQueryGraphProviderExecution for WorkflowAbandonExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        step.perform_work_unit(|| Ok(()))?;
        step.emit_projection_chunk(graph_material())
            .map_err(step_failure)?;
        let disposition = if self.step_ordinal == 0 {
            WorthQueryGraphProviderStepDisposition::continue_work()
        } else {
            WorthQueryGraphProviderStepDisposition::complete("workflow-abandon")
                .map_err(WorthQueryGraphProviderFailure::new)?
        };
        self.step_ordinal = self.step_ordinal.saturating_add(1);
        Ok(disposition)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for WorkflowAbandonProvider {
    type Execution = WorkflowAbandonExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "workflow-abandon",
            8,
        )
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        admit_provider_execution(start, WorkflowAbandonExecution { step_ordinal: 0 })
    }
}

#[test]
fn workflow_active_abandonment_releases_provider_execution() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = workflow_abandon_world(resource_request);
        let active =
            begin_workflow_projection(execution, running, &graph, "workflow-active-abandon");
        let terminal = failed_terminal(active.abandon());
        assert_eq!(terminal.provider_work().abandoned_call_count(), 1);
        assert_eq!(terminal.provider_work().retained_bytes(), 0);
        assert_eq!(
            terminal
                .provider_work()
                .provider_execution_release()
                .release_count(),
            1
        );
    });
}

#[test]
fn workflow_pending_and_paused_abandonment_release_output_and_queue() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = workflow_abandon_world(resource_request);
        let active =
            begin_workflow_projection(execution, running, &graph, "workflow-pending-abandon");
        let pending = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::ChunkReady(pending) => pending,
            _ => panic!("workflow provider did not expose its first bounded chunk"),
        };
        let terminal = failed_terminal(pending.abandon());
        assert_eq!(terminal.provider_work().queue_state_mutation_count(), 2);
        assert_eq!(terminal.provider_work().retained_bytes(), 0);
        assert!(terminal.provider_work().peak_retained_bytes() > 0);

        let (running, graph) = workflow_abandon_world(resource_request);
        let active =
            begin_workflow_projection(execution, running, &graph, "workflow-paused-abandon");
        let pending = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::ChunkReady(pending) => pending,
            _ => panic!("workflow provider did not expose its first bounded chunk"),
        };
        let paused = match pending.acknowledge() {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow chunk acknowledgement did not reach a paused safe point"),
        };
        let terminal = failed_terminal(paused.abandon());
        assert_eq!(terminal.provider_work().abandoned_call_count(), 1);
        assert_eq!(terminal.provider_work().retained_bytes(), 0);
    });
}

fn workflow_abandon_world(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::domain_computation::WorthQueryRunningWorkflowRun,
    WorthQueryInstalledGraphParticipationAuthority,
) {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            WorkflowAbandonProvider,
        ),
    );
    let provider_support = anchor.resource_support().clone();
    let graph = super::workflow_provider_steps::installed_graph(
        &installer,
        "workflow-abandon-graph",
        anchor,
    );
    let runtime =
        super::workflow_provider_steps::installed_runtime(installer, "workflow abandonment");
    let operation_resources = admitted_plan("workflow-abandon-operation", 8);
    let stage_resources = admitted_plan_with_graph_support(
        "workflow-abandon-operation:stage",
        8,
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
    let running = super::workflow_provider_steps::admitted_workflow(
        &runtime,
        &operation,
        resources,
        resource_request,
    );
    (running, graph)
}

fn begin_workflow_projection(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    running: crate::domain_computation::WorthQueryRunningWorkflowRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    identity: &str,
) -> crate::domain_computation::WorthQueryActiveWorkflowGraphExecution {
    running
        .begin_stage_graph_execution(
            execution,
            "stage",
            graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Project,
                identity,
            ),
        )
        .expect("workflow abandonment provider should start")
}

fn failed_terminal(
    outcome: WorthQueryWorkflowGraphStepOutcome,
) -> crate::domain_computation::WorthQueryWorkflowRunTerminal {
    match outcome {
        WorthQueryWorkflowGraphStepOutcome::Failed(terminal) => terminal,
        _ => panic!("explicit workflow abandonment did not produce a failed terminal"),
    }
}

fn step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
