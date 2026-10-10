use super::*;
use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode, WorthQueryResourceDimension,
    WorthQueryResourceLimitRequest, WorthQuerySemanticScaleRequest,
};
use worth_query_installation::facade::WorthQueryExecutionResourceEnvelope;

struct StageQueueContractProvider {
    begins: Arc<AtomicUsize>,
    advances: Arc<AtomicUsize>,
}

struct StageQueueContractExecution {
    advances: Arc<AtomicUsize>,
}

impl WorthQueryGraphProviderExecution for StageQueueContractExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        self.advances.fetch_add(1, Ordering::Relaxed);
        step.perform_work_unit(|| Ok(()))?;
        step.emit_projection_chunk(graph_material_rows(4))
            .map_err(step_failure)?;
        WorthQueryGraphProviderStepDisposition::complete("stage-queue-contract")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for StageQueueContractProvider {
    type Execution = StageQueueContractExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support_for_envelope(
            "stage-queue-contract-provider",
            WorthQueryExecutionResourceEnvelope::new(
                WorthQuerySemanticScaleRequest::bounded(4),
                WorthQueryResourceLimitRequest::bounded(4)
                    .with(WorthQueryResourceDimension::QueueDepth, 8)
                    .with(WorthQueryResourceDimension::RetainedBytes, 4_096),
                WorthQueryExecutionMode::Synchronous,
                None,
                WorthQueryCancellationSafePointFamily::new("execution-chunk").unwrap(),
            ),
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
        self.begins.fetch_add(1, Ordering::Relaxed);
        admit_provider_execution(
            start,
            StageQueueContractExecution {
                advances: Arc::clone(&self.advances),
            },
        )
    }
}

#[test]
fn stage_contract_wider_than_the_signal_queue_denies_before_provider_construction() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let begins = Arc::new(AtomicUsize::new(0));
        let advances = Arc::new(AtomicUsize::new(0));
        let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            StageQueueContractProvider {
                begins: Arc::clone(&begins),
                advances: Arc::clone(&advances),
            },
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = super::workflow_provider_steps::installed_graph(
            &installer,
            "stage-queue-contract-graph",
            provider_anchor,
        );
        let runtime =
            super::workflow_provider_steps::installed_runtime(installer, "stage queue contract");
        let operation_resources = admitted_plan("stage-queue-contract", 4);
        let stage_resources = admitted_plan_with_graph_support(
            "stage-queue-contract:stage",
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
        let running = super::workflow_provider_steps::admitted_workflow(
            &runtime,
            &operation,
            resources,
            resource_request,
        );
        let failure = match running.begin_stage_graph_execution(
            execution,
            "stage",
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Project,
                "stage-queue-contract",
            ),
        ) {
            Ok(_) => panic!("a stage contract wider than Signal started its provider"),
            Err(failure) => failure,
        };
        assert_eq!(
        failure.kind(),
        crate::domain_computation::WorthQueryWorkflowGraphExecutionStartFailureKind::StepContract(
            crate::domain_computation::WorthQueryManagedStepContractDenialKind::QueueDepthExceeded,
        )
    );
        assert_eq!(begins.load(Ordering::Relaxed), 0);
        assert_eq!(advances.load(Ordering::Relaxed), 0);
        let terminal = failure
            .into_running()
            .terminal(WorthQueryManagedRunTerminalKind::Cancelled);
        assert_eq!(terminal.provider_work().provider_step_attempt_count(), 0);
        assert_eq!(
            terminal
                .provider_work()
                .output_capacity_classification_count(),
            0
        );
        assert_eq!(terminal.provider_work().completed_work_units(), 0);
        assert_eq!(terminal.provider_work().retained_bytes(), 0);
        match terminal.cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {}
            _ => panic!("contract-denied workflow should clean up"),
        }
    });
}

fn step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
