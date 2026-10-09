#[cfg(feature = "allocation-probes")]
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

const UNRELATED_WIDTH: usize = 12;

struct YieldCostProvider {
    suspension_count: Arc<AtomicUsize>,
    retained_probe_count: Arc<AtomicUsize>,
}

struct YieldCostExecution {
    suspension_count: Arc<AtomicUsize>,
    retained_probe_count: Arc<AtomicUsize>,
    retained: Option<WorthQueryGraphProviderRetainedMemory>,
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
        Box<dyn crate::domain_computation::WorthQueryGraphProviderCheckpoint>,
        WorthQueryGraphProviderFailure,
    > {
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
        crate::domain_computation::provider_session::execution_resource_support_with_yield(
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

struct YieldCostCheckpoint {
    retained_probe_count: Arc<AtomicUsize>,
    retained: WorthQueryGraphProviderRetainedMemory,
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint for YieldCostCheckpoint {
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
struct YieldCostEvidence {
    provider_steps: usize,
    safe_point_lookups: usize,
    pressure_classifications: usize,
    output_capacity_classifications: usize,
    suspension_count: usize,
    retained_probe_count: usize,
    retained_capacity_count: usize,
    yield_counters: crate::domain_computation::WorthQueryYieldTransitionCounters,
}

#[test]
fn yield_transition_work_is_invariant_to_unrelated_live_authority_width() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let baseline = execute_target(execution, 0);
        let wide = execute_target(execution, UNRELATED_WIDTH);
        assert_eq!(baseline, wide);
        assert_eq!(
            baseline,
            YieldCostEvidence {
                provider_steps: 1,
                safe_point_lookups: 2,
                pressure_classifications: 2,
                output_capacity_classifications: 0,
                suspension_count: 1,
                retained_probe_count: 1,
                retained_capacity_count: 2,
                yield_counters: expected_direct_yield_counters(),
            }
        );
    });
}

#[test]
#[cfg(feature = "allocation-probes")]
fn yield_transition_allocation_count_is_width_invariant() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("isolated_yield_transition_allocation_slope_probe")
        .env("WORTH_QUERY_YIELD_ALLOCATION_PROBE", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "yield allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
#[cfg(feature = "allocation-probes")]
fn isolated_yield_transition_allocation_slope_probe() {
    if std::env::var_os("WORTH_QUERY_YIELD_ALLOCATION_PROBE").is_none() {
        return;
    }
    let baseline = measured_target(0);
    let wide = measured_target(UNRELATED_WIDTH);
    assert_eq!(baseline.allocations, wide.allocations);
    assert_eq!(baseline.reallocations, wide.reallocations);
}

#[test]
#[cfg(feature = "allocation-probes")]
fn workflow_yield_transition_allocation_is_unrelated_authority_invariant() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("isolated_workflow_yield_transition_allocation_slope_probe")
        .env("WORTH_QUERY_WORKFLOW_YIELD_ALLOCATION_PROBE", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "workflow yield allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn execute_target(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    unrelated_width: usize,
) -> YieldCostEvidence {
    let (paused, unrelated, suspension_count, retained_probe_count) =
        prepared_target(execution, unrelated_width);
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

#[cfg(feature = "allocation-probes")]
mod allocation_measurement;
#[cfg(feature = "allocation-probes")]
use allocation_measurement::{measured_target, measured_workflow_target};

fn prepared_target(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    unrelated_width: usize,
) -> (
    crate::domain_computation::WorthQueryPausedDirectGraphExecution,
    Vec<(
        crate::domain_computation::WorthQueryRunningWorkflowRun,
        crate::domain_computation::WorthQueryMoveOnlyArtifactHandle,
    )>,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
) {
    let disposed = Arc::new(AtomicUsize::new(0));
    let unrelated = (0..unrelated_width)
        .map(|index| super::cost_bound::unrelated_artifact_run(index, Arc::clone(&disposed)))
        .collect();
    let suspension_count = Arc::new(AtomicUsize::new(0));
    let retained_probe_count = Arc::new(AtomicUsize::new(0));
    let (running, graph) = managed_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Observe,
        YieldCostProvider {
            suspension_count: Arc::clone(&suspension_count),
            retained_probe_count: Arc::clone(&retained_probe_count),
        },
    );
    let active = running
        .begin_graph_execution(
            execution,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "yield-cost-slope",
            ),
        )
        .expect("yield cost provider should begin");
    let paused = match active.advance(execution) {
        WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("yield cost provider did not pause"),
    };
    (paused, unrelated, suspension_count, retained_probe_count)
}

#[cfg(feature = "allocation-probes")]
fn prepared_workflow_target(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    unrelated_width: usize,
) -> (
    crate::domain_computation::WorthQueryPausedWorkflowGraphExecution,
    Vec<(
        crate::domain_computation::WorthQueryRunningWorkflowRun,
        crate::domain_computation::WorthQueryMoveOnlyArtifactHandle,
    )>,
) {
    let disposed = Arc::new(AtomicUsize::new(0));
    let unrelated = (0..unrelated_width)
        .map(|index| super::cost_bound::unrelated_artifact_run(index, Arc::clone(&disposed)))
        .collect();
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            YieldCostProvider {
                suspension_count: Arc::new(AtomicUsize::new(0)),
                retained_probe_count: Arc::new(AtomicUsize::new(0)),
            },
        ),
    );
    let provider_support = provider_anchor.resource_support().clone();
    let graph = super::workflow_provider_steps::installed_graph(
        &installer,
        "workflow-yield-cost-graph",
        provider_anchor,
    );
    let runtime =
        super::workflow_provider_steps::installed_runtime(installer, "workflow yield cost");
    let operation_resources =
        crate::domain_computation::provider_session::admitted_yield_plan("workflow-yield-cost", 8);
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
    let running =
        super::workflow_provider_steps::admitted_workflow(&runtime, &operation, resources);
    let active = running
        .begin_stage_graph_execution(
            execution,
            "stage",
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "workflow-yield-cost",
            ),
        )
        .expect("workflow yield cost provider should begin");
    let paused = match active.advance(execution) {
        WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("workflow yield cost provider did not pause"),
    };
    (paused, unrelated)
}

fn yield_target(
    paused: crate::domain_computation::WorthQueryPausedDirectGraphExecution,
) -> crate::domain_computation::WorthQueryYieldedDirectRun {
    match paused.yield_run() {
        crate::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("eligible cost target did not yield"),
    }
}

fn step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}

fn expected_direct_yield_counters() -> crate::domain_computation::WorthQueryYieldTransitionCounters
{
    let mut counters = crate::domain_computation::WorthQueryYieldTransitionCounters::default();
    counters.classified_eligibility();
    counters.attempted_bridge_finalization();
    counters.attempted_checkpoint_suspension();
    counters.observed_checkpoint_retained_bytes(1);
    counters.validated_retained_resources();
    counters.minted_yielded_capability();
    counters
}

#[cfg(feature = "allocation-probes")]
fn expected_workflow_yield_counters() -> crate::domain_computation::WorthQueryYieldTransitionCounters
{
    let mut counters = expected_direct_yield_counters();
    counters.observed_artifact_registry();
    counters.observed_artifact_registry();
    counters.validated_retained_resources();
    counters
}

#[path = "yield_cost_bound/isolated_workflow_yield_transition_allocation_slope_probe.rs"]
mod isolated_workflow_yield_transition_allocation_slope_probe;
