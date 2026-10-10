use super::*;

struct SupportAffinityProvider {
    begins: Arc<AtomicUsize>,
}

struct SupportAffinityExecution;

struct ForeignSafePointProvider {
    begins: Arc<AtomicUsize>,
}

impl WorthQueryGraphProviderExecution for SupportAffinityExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        step.perform_work_unit(|| Ok(()))?;
        WorthQueryGraphProviderStepDisposition::complete("support-affinity")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for SupportAffinityProvider {
    type Execution = SupportAffinityExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "support-affinity",
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
        self.begins.fetch_add(1, Ordering::Relaxed);
        admit_provider_execution(start, SupportAffinityExecution)
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for ForeignSafePointProvider {
    type Execution = SupportAffinityExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        use worth_query_declaration::facade::domain_computation::{
            WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode,
            WorthQueryResourceDimension, WorthQueryResourceLimitRequest,
            WorthQuerySemanticScaleRequest,
        };

        crate::domain_computation::provider_session::execution_resource_support_for_envelope(
            "foreign-safe-point-provider",
            worth_query_installation::facade::WorthQueryExecutionResourceEnvelope::new(
                WorthQuerySemanticScaleRequest::bounded(8),
                WorthQueryResourceLimitRequest::bounded(8)
                    .with(WorthQueryResourceDimension::RetainedBytes, 4_096),
                WorthQueryExecutionMode::Synchronous,
                None,
                WorthQueryCancellationSafePointFamily::new("foreign-row-boundary").unwrap(),
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
        admit_provider_execution(start, SupportAffinityExecution)
    }
}

#[test]
fn compatible_but_independently_minted_provider_support_denies_before_provider_start() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let begins = Arc::new(AtomicUsize::new(0));
        let (running, graph, _, _runtime) = managed_graph_run_with_provider_and_admitted_support(
            SupportAffinityProvider {
                begins: Arc::clone(&begins),
            },
            ManagedGraphRunConfiguration {
                access: WorthQueryOperationGraphAccess::Observe,
                touch: false,
                binding_identity: "managed-graph-binding",
            },
            independently_minted_support,
            resource_request,
        );
        let failure = match running.begin_graph_execution(
            execution,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "foreign-provider-support",
            ),
        ) {
            Ok(_) => panic!("independently minted provider support entered the managed lane"),
            Err(failure) => failure,
        };
        assert_eq!(
        failure.kind(),
        crate::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::ProviderSupportMismatch
    );
        assert_eq!(begins.load(Ordering::Relaxed), 0);

        let terminal = failure
            .into_running()
            .completed()
            .expect("support mismatch must leave the running authority usable");
        terminal
            .cleanup()
            .expect("support mismatch must preserve exact cleanup authority");
    });
}

#[test]
fn workflow_stage_safe_point_family_must_match_the_running_bridge_basis() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph, begins) = foreign_safe_point_workflow(resource_request);
        let failure = match running.begin_stage_graph_execution(
            execution,
            "stage",
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "foreign-safe-point",
            ),
        ) {
            Ok(_) => panic!("foreign stage safe-point family entered the managed lane"),
            Err(failure) => failure,
        };
        assert_eq!(
        failure.kind(),
        crate::domain_computation::WorthQueryWorkflowGraphExecutionStartFailureKind::StepContract(
            crate::domain_computation::WorthQueryManagedStepContractDenialKind::SafePointFamilyMismatch,
        )
    );
        assert_eq!(begins.load(Ordering::Relaxed), 0);
        let terminal = failure
            .into_running()
            .completed()
            .expect("safe-point mismatch must preserve the running authority");
        match terminal.cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {}
            _ => panic!("safe-point mismatch must preserve cleanup authority"),
        }
    });
}

fn foreign_safe_point_workflow(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::domain_computation::WorthQueryRunningWorkflowRun,
    WorthQueryInstalledGraphParticipationAuthority,
    Arc<AtomicUsize>,
) {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let begins = Arc::new(AtomicUsize::new(0));
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            ForeignSafePointProvider {
                begins: Arc::clone(&begins),
            },
        ),
    );
    let provider_support = provider_anchor.resource_support().clone();
    let graph = super::workflow_provider_steps::installed_graph(
        &installer,
        "foreign-safe-point-graph",
        provider_anchor,
    );
    let runtime =
        super::workflow_provider_steps::installed_runtime(installer, "foreign safe point");
    let operation_resources = admitted_plan("foreign-safe-point", 8);
    let stage_resources = admitted_plan_with_graph_support(
        "foreign-safe-point:stage",
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
        WorthQueryOperationGraphAccess::Observe,
    );
    let running = super::workflow_provider_steps::admitted_workflow(
        &runtime,
        &operation,
        resources,
        resource_request,
    );
    (running, graph, begins)
}

fn independently_minted_support(
    exact: &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
    let capacity =
        worth_query_admission::facade::resource_admission::WorthQueryFixedExecutionCapacity::new(
            exact.capacity_subject_identity(),
            8,
        )
        .expect("fixture capacity should remain valid");
    let substitute =
        worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport::new(
            exact.provider().clone(),
            exact.access_product().clone(),
            exact.allocator().clone(),
            exact.envelope().clone(),
            Arc::new(capacity),
        );
    assert_eq!(substitute.identity(), exact.identity());
    assert_ne!(substitute, *exact);
    substitute
}
