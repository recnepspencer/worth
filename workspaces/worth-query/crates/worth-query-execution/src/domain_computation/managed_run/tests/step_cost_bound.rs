use super::*;

const UNRELATED_WIDTH: usize = 12;

struct CostSlopeProvider {
    advances: Arc<AtomicUsize>,
    chunk_count: usize,
}

struct CostSlopeExecution {
    advances: Arc<AtomicUsize>,
    remaining_chunks: usize,
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
        crate::domain_computation::provider_session::execution_resource_support(
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
struct StepCostEvidence {
    issued_calls: usize,
    admitted_receipts: usize,
    completed_work_units: u64,
    provider_steps: usize,
    safe_point_lookups: usize,
    pressure_classifications: usize,
    output_capacity_classifications: usize,
    queue_lookups: usize,
    queue_mutations: usize,
    retained_bytes: usize,
}

#[test]
fn one_provider_step_has_constant_work_under_unrelated_authority_width() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let baseline = execute_target(execution, 0, resource_request);
        let wide = execute_target(execution, UNRELATED_WIDTH, resource_request);
        assert_eq!(baseline, wide);
        assert_eq!(
            baseline,
            StepCostEvidence {
                issued_calls: 1,
                admitted_receipts: 1,
                completed_work_units: 1,
                provider_steps: 1,
                safe_point_lookups: 3,
                pressure_classifications: 3,
                output_capacity_classifications: 1,
                queue_lookups: 2,
                queue_mutations: 2,
                retained_bytes: 0,
            }
        );
    });
}

#[test]
fn admitted_chunk_count_has_only_the_declared_linear_step_cost() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let one_chunk = execute_target_with_chunks(execution, 0, 1, resource_request);
        let four_chunks = execute_target_with_chunks(execution, 0, 4, resource_request);
        assert_eq!(
            one_chunk,
            StepCostEvidence {
                issued_calls: 1,
                admitted_receipts: 1,
                completed_work_units: 1,
                provider_steps: 1,
                safe_point_lookups: 3,
                pressure_classifications: 3,
                output_capacity_classifications: 1,
                queue_lookups: 2,
                queue_mutations: 2,
                retained_bytes: 0,
            }
        );
        assert_eq!(
            four_chunks,
            StepCostEvidence {
                issued_calls: 1,
                admitted_receipts: 1,
                completed_work_units: 4,
                provider_steps: 4,
                safe_point_lookups: 12,
                pressure_classifications: 12,
                output_capacity_classifications: 4,
                queue_lookups: 8,
                queue_mutations: 8,
                retained_bytes: 0,
            }
        );
    });
}

#[test]
#[cfg(feature = "allocation-probes")]
fn one_provider_step_allocation_count_is_width_invariant() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("isolated_provider_step_allocation_slope_probe")
        .env("WORTH_QUERY_STEP_ALLOCATION_PROBE", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "provider-step allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
#[cfg(feature = "allocation-probes")]
fn isolated_provider_step_allocation_slope_probe() {
    if std::env::var_os("WORTH_QUERY_STEP_ALLOCATION_PROBE").is_none() {
        return;
    }
    let baseline = measured_target(0);
    let wide = measured_target(UNRELATED_WIDTH);
    assert_eq!(baseline.allocations, wide.allocations);
    assert_eq!(baseline.reallocations, wide.reallocations);
}

fn execute_target(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    unrelated_width: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> StepCostEvidence {
    execute_target_with_chunks(execution, unrelated_width, 1, resource_request)
}

fn execute_target_with_chunks(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

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
    super::cost_bound::assert_exact_admission_work(terminal.counters());
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

#[cfg(feature = "allocation-probes")]
fn measured_target(unrelated_width: usize) -> stats_alloc::Stats {
    crate::domain_computation::primary_graph::with_test_advancement(|execution| {
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

fn prepared_target(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    unrelated_width: usize,
    admitted_chunk_count: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::domain_computation::WorthQueryActiveDirectGraphExecution,
    Vec<(
        crate::domain_computation::WorthQueryRunningWorkflowRun,
        crate::domain_computation::WorthQueryMoveOnlyArtifactHandle,
    )>,
    Arc<AtomicUsize>,
) {
    let disposed = Arc::new(AtomicUsize::new(0));
    let unrelated = (0..unrelated_width)
        .map(|index| {
            super::cost_bound::unrelated_artifact_run(
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
    let active = running
        .begin_graph_execution(
            execution,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Project,
                "cost-slope",
            ),
        )
        .expect("cost-slope provider should start");
    (active, unrelated, advances)
}

fn complete_target(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    active: crate::domain_computation::WorthQueryActiveDirectGraphExecution,
) -> crate::domain_computation::WorthQueryCompletedDirectGraphExecution {
    let mut outcome = active.advance(execution);
    loop {
        outcome = match outcome {
            WorthQueryDirectGraphStepOutcome::ChunkReady(pending) => pending.acknowledge(),
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused.advance(execution),
            WorthQueryDirectGraphStepOutcome::Completed(completion) => return completion,
            _ => panic!("cost-slope provider left the admitted chunk progression"),
        };
    }
}

fn step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
