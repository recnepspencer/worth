pub use crate::domain_computation::managed_run::tests::*;

pub const UNRELATED_WIDTH: usize = 12;

pub struct CostSlopeProvider {
    pub advances: Arc<AtomicUsize>,
    pub chunk_count: usize,
}

pub struct CostSlopeExecution {
    pub advances: Arc<AtomicUsize>,
    pub remaining_chunks: usize,
}

impl WorthQueryGraphProviderExecution for CostSlopeExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        self.advances.fetch_add(1, Ordering::Relaxed);
        step.perform_work_unit(|| Ok(()))?;
        step.emit_projection_chunk(graph_material())
            .map_err(step_failure)?;
        self.remaining_chunks = self
            .remaining_chunks
            .checked_sub(1)
            .expect("cost provider cannot advance after its declared chunks");
        if self.remaining_chunks == 0 {
            WorthQueryGraphProviderStepDisposition::complete("cost-slope")
                .map_err(WorthQueryGraphProviderFailure::new)
        } else {
            Ok(WorthQueryGraphProviderStepDisposition::continue_work())
        }
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for CostSlopeProvider {
    type Execution = CostSlopeExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::execution_resource_support(
            "cost-slope-provider",
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
            CostSlopeExecution {
                advances: Arc::clone(&self.advances),
                remaining_chunks: self.chunk_count,
            },
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StepCostEvidence {
    pub issued_calls: usize,
    pub admitted_receipts: usize,
    pub completed_work_units: u64,
    pub provider_steps: usize,
    pub safe_point_lookups: usize,
    pub pressure_classifications: usize,
    pub output_capacity_classifications: usize,
    pub queue_lookups: usize,
    pub queue_mutations: usize,
    pub retained_bytes: usize,
}

pub fn execute_target(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    unrelated_width: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> StepCostEvidence {
    execute_target_with_chunks(execution, unrelated_width, 1, resource_request)
}

pub fn execute_target_with_chunks(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    unrelated_width: usize,
    admitted_chunk_count: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> StepCostEvidence {
    let (active, unrelated, advances) = prepared_target(
        execution,
        unrelated_width,
        admitted_chunk_count,
        resource_request,
    );
    let completion = complete_target(execution, active);
    assert_eq!(advances.load(Ordering::Relaxed), admitted_chunk_count);
    let terminal = completion.into_running().completed().unwrap();
    worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::cost_bound::assert_exact_admission_work(terminal.counters());
    let work = terminal.provider_work();
    let evidence = StepCostEvidence {
        issued_calls: work.issued_call_count(),
        admitted_receipts: work.admitted_receipt_count(),
        completed_work_units: work.completed_work_units(),
        provider_steps: work.provider_step_attempt_count(),
        safe_point_lookups: work.safe_point_request_lookup_count(),
        pressure_classifications: work.pressure_classification_count(),
        output_capacity_classifications: work.output_capacity_classification_count(),
        queue_lookups: work.queue_request_lookup_count(),
        queue_mutations: work.queue_state_mutation_count(),
        retained_bytes: work.retained_bytes(),
    };
    terminal.cleanup().expect("cost target should clean up");
    drop(unrelated);
    evidence
}

pub fn measured_target(unrelated_width: usize) -> stats_alloc::Stats {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|execution| {
        let bootstrap = execution.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();
        let execution = &execution;
        let (active, unrelated, _) =
            prepared_target(execution, unrelated_width, 1, resource_request);
        let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
        let completion = complete_target(execution, active);
        let stats = region.change();
        completion
            .into_running()
            .completed()
            .unwrap()
            .cleanup()
            .expect("measured target should clean up");
        drop(unrelated);
        stats
    })
}

pub fn prepared_target(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    unrelated_width: usize,
    admitted_chunk_count: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryActiveDirectGraphExecution,
    Vec<(
        worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun,
        worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle,
    )>,
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
    let advances = Arc::new(AtomicUsize::new(0));
    let (running, graph) = managed_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Project,
        CostSlopeProvider {
            advances: Arc::clone(&advances),
            chunk_count: admitted_chunk_count,
        },
        resource_request,
    );
    let active = {
        let installed_fixture_phase = running.fixture_phase(execution);
        running.begin_graph_execution(
            &installed_fixture_phase,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Project,
                "cost-slope",
            ),
        )
    }
    .expect("cost-slope provider should start");
    (active, unrelated, advances)
}

pub fn complete_target(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    active: worth_query_execution::facade::domain_computation::WorthQueryActiveDirectGraphExecution,
) -> worth_query_execution::facade::domain_computation::WorthQueryCompletedDirectGraphExecution {
    let mut outcome = {
        let installed_fixture_phase = active.fixture_phase(execution);
        active.advance(&installed_fixture_phase)
    };
    loop {
        outcome = match outcome {
            WorthQueryDirectGraphStepOutcome::ChunkReady(pending) => pending.acknowledge(),
            WorthQueryDirectGraphStepOutcome::Continue(paused) => {
                let installed_fixture_phase = paused.fixture_phase(execution);
                paused.advance(&installed_fixture_phase)
            }
            WorthQueryDirectGraphStepOutcome::Completed(completion) => return completion,
            _ => panic!("cost-slope provider left the admitted chunk progression"),
        };
    }
}

pub fn step_failure(
    denial: worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
