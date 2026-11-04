pub use crate::domain_computation::managed_run::tests::yield_cost_bound::expected_counters::expected_direct_yield_counters;
pub use std::collections::BTreeMap;
pub use std::sync::atomic::AtomicUsize;
pub use std::sync::atomic::Ordering;

pub use crate::domain_computation::managed_run::tests::*;

pub const UNRELATED_WIDTH: usize = 12;

pub struct YieldCostProvider {
    pub suspension_count: Arc<AtomicUsize>,
    pub retained_probe_count: Arc<AtomicUsize>,
}

pub struct YieldCostExecution {
    pub suspension_count: Arc<AtomicUsize>,
    pub retained_probe_count: Arc<AtomicUsize>,
    pub retained: Option<WorthQueryGraphProviderRetainedMemory>,
}

impl WorthQueryGraphProviderExecution for YieldCostExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        step.perform_work_unit(|| Ok(()))?;
        self.retained = Some(step.retain_bytes(3).map_err(step_failure)?);
        step.record_checkpoint_available().map_err(step_failure)?;
        Ok(WorthQueryGraphProviderStepDisposition::continue_work())
    }

    fn suspend(
        &mut self,
    ) -> Result<
        Box<dyn worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint>,
        WorthQueryGraphProviderFailure,
    >{
        self.suspension_count.fetch_add(1, Ordering::Relaxed);
        Ok(Box::new(YieldCostCheckpoint {
            retained_probe_count: Arc::clone(&self.retained_probe_count),
            retained: self
                .retained
                .take()
                .expect("yield cost execution transfers governed retained memory"),
        }))
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for YieldCostProvider {
    type Execution = YieldCostExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::execution_resource_support_with_yield(
            "yield-cost-provider",
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
        admit_provider_execution(
            start,
            YieldCostExecution {
                suspension_count: Arc::clone(&self.suspension_count),
                retained_probe_count: Arc::clone(&self.retained_probe_count),
                retained: None,
            },
        )
    }
}

pub struct YieldCostCheckpoint {
    pub retained_probe_count: Arc<AtomicUsize>,
    pub retained: WorthQueryGraphProviderRetainedMemory,
}

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
    for YieldCostCheckpoint
{
    fn retained_bytes(&self) -> u64 {
        self.retained_probe_count.fetch_add(1, Ordering::Relaxed);
        u64::try_from(self.retained.len()).unwrap()
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        _memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        Err(WorthQueryGraphProviderFailure::new(
            "Phase 6.3 cost probe must never restore its checkpoint",
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct YieldCostEvidence {
    pub provider_steps: usize,
    pub safe_point_lookups: usize,
    pub pressure_classifications: usize,
    pub output_capacity_classifications: usize,
    pub suspension_count: usize,
    pub retained_probe_count: usize,
    pub retained_capacity_count: usize,
    pub yield_counters:
        worth_query_execution::facade::domain_computation::WorthQueryYieldTransitionCounters,
}

pub fn execute_target(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    unrelated_width: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> YieldCostEvidence {
    let (paused, unrelated, suspension_count, retained_probe_count) =
        prepared_target(execution, unrelated_width, resource_request);
    let yielded = yield_target(paused);
    let work = yielded.inspection().provider_work();
    let evidence = YieldCostEvidence {
        provider_steps: work.provider_step_attempt_count(),
        safe_point_lookups: work.safe_point_request_lookup_count(),
        pressure_classifications: work.pressure_classification_count(),
        output_capacity_classifications: work.output_capacity_classification_count(),
        suspension_count: suspension_count.load(Ordering::Relaxed),
        retained_probe_count: retained_probe_count.load(Ordering::Relaxed),
        retained_capacity_count: yielded.inspection().retained_capacity_reservation_count(),
        yield_counters: yielded.inspection().yield_counters(),
    };
    let _ = yielded.cleanup();
    drop(unrelated);
    evidence
}

pub use crate::domain_computation::managed_run::tests::yield_cost_bound::allocation_measurement::measured_target;
pub use crate::domain_computation::managed_run::tests::yield_cost_bound::allocation_measurement::measured_workflow_target;

pub fn prepared_target(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    unrelated_width: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryPausedDirectGraphExecution,
    Vec<(
        worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun,
        worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle,
    )>,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
) {
    let disposed = Arc::new(AtomicUsize::new(0));
    let unrelated = (0..unrelated_width)
        .map(|index| {
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::unrelated_artifact_run(
                index,
                Arc::clone(&disposed),
                resource_request,
            )
        })
        .collect();
    let suspension_count = Arc::new(AtomicUsize::new(0));
    let retained_probe_count = Arc::new(AtomicUsize::new(0));
    let (running, graph) = managed_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        YieldCostProvider {
            suspension_count: Arc::clone(&suspension_count),
            retained_probe_count: Arc::clone(&retained_probe_count),
        },
        resource_request,
    );
    let active = {
        let installed_fixture_phase = running.fixture_phase_for_test(execution);
        running.begin_graph_execution(
            &installed_fixture_phase,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "yield-cost-slope",
            ),
        )
    }
    .expect("yield cost provider should begin");
    let paused = {
        let fixture_phase_result = {
            let installed_fixture_phase = active.fixture_phase_for_test(execution);
            active.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("yield cost provider did not pause"),
        }
    };
    (paused, unrelated, suspension_count, retained_probe_count)
}

pub fn prepared_workflow_target(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    unrelated_width: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryPausedWorkflowGraphExecution,
    Vec<(
        worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun,
        worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle,
    )>,
) {
    let disposed = Arc::new(AtomicUsize::new(0));
    let unrelated = (0..unrelated_width)
        .map(|index| {
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::unrelated_artifact_run(
                index,
                Arc::clone(&disposed),
                resource_request,
            )
        })
        .collect();
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install::<
            ManagedGraph,
            _,
        >(YieldCostProvider {
            suspension_count: Arc::new(AtomicUsize::new(0)),
            retained_probe_count: Arc::new(AtomicUsize::new(0)),
        }),
    );
    let provider_support = provider_anchor.resource_support().clone();
    let graph = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_graph(
        &installer,
        "workflow-yield-cost-graph",
        provider_anchor,
    );
    let runtime =
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_runtime(installer, "workflow yield cost");
    let operation_resources =
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::admitted_yield_plan("workflow-yield-cost", 8);
    let stage_resources = admitted_plan_with_graph_support(
        "workflow-yield-cost:stage",
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
    let running = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::admitted_workflow(
        &runtime,
        &operation,
        resources,
        resource_request,
    );
    let installed_fixture_phase = running.fixture_phase_for_test(execution);
    let active = running
        .begin_stage_graph_execution(
            &installed_fixture_phase,
            "stage",
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "workflow-yield-cost",
            ),
        )
        .expect("workflow yield cost provider should begin");
    let paused = match active.advance(&installed_fixture_phase) {
        WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("workflow yield cost provider did not pause"),
    };
    (paused, unrelated)
}

pub fn yield_target(
    paused: worth_query_execution::facade::domain_computation::WorthQueryPausedDirectGraphExecution,
) -> worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRun {
    match paused.yield_run() {
        worth_query_execution::facade::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("eligible cost target did not yield"),
    }
}

pub fn step_failure(
    denial: worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
