mod application;
mod counters;
mod evidence;
mod planning;
mod seeds;

use std::ops::DerefMut;

#[cfg(any(test, doctest))]
use crate::data::aspect::Aspect;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
#[cfg(any(test, doctest))]
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
#[cfg(any(test, doctest))]
use crate::data::output::ChangedRegion;
use crate::data::proof::{
    DirtyBatch, FrontierPlan, InvalidationSeed, InvalidationTraceRecord, SourceRecomputeAdmission,
};
use crate::diagnostics::failure::{ExecutionFailureContext, ExecutionFailurePhase};
use crate::diagnostics::lineage::InvalidationCause;
use crate::diagnostics::recorder::{record_invalidation_lineage, DiagnosticsRecorder};

#[cfg(any(test, doctest))]
pub fn mark_dirty(
    mut graph: impl DerefMut<Target = SignalGraph>,
    source: NodeId,
    changed_aspect: Aspect,
) -> Result<(), SignalError> {
    let _ = mark_dirty_batch(
        graph.deref_mut(),
        &DirtyBatch::singleton(source, changed_aspect, Vec::<ChangedRegion>::new()),
    )?;
    Ok(())
}

#[cfg(any(test, doctest))]
pub fn mark_dirty_with_regions(
    mut graph: impl DerefMut<Target = SignalGraph>,
    source: NodeId,
    changed_aspect: Aspect,
    changed_regions: &[ChangedRegion],
) -> Result<(), SignalError> {
    let _ = mark_dirty_batch(
        graph.deref_mut(),
        &DirtyBatch::singleton(source, changed_aspect, changed_regions.to_vec()),
    )?;
    Ok(())
}

pub fn mark_dirty_batch(
    mut graph: impl DerefMut<Target = SignalGraph>,
    dirty: &DirtyBatch,
) -> Result<SourceRecomputeAdmission, SignalError> {
    let graph = graph.deref_mut();
    let batch_width = dirty.as_slice().len() as u64;
    graph.with_telemetry(|telemetry| telemetry.invalidation.batch_width += batch_width);
    let admission = SourceRecomputeAdmission::new(dirty.clone());
    for seed in &admission.seeds {
        super::causality::source_seed::validate_source_seed(graph, seed)?;
    }
    for entry in dirty.as_slice() {
        graph.note_change_input(
            entry.source,
            entry.changed_aspect,
            entry.changed_regions.as_slice(),
        );
    }

    let result = (|| {
        let plan = plan_invalidation_frontier(graph, dirty)?;
        let capture_frontier = graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::FrontierTrace,
        );
        let Some(mut summary) = execute_invalidation_frontier(graph, &plan, capture_frontier)?
        else {
            return Ok(());
        };
        let trace_records = retained_trace_records(graph, &plan)?;
        summary.counters.frontier_trace_retained_count = trace_records.len() as u64;
        let partition_checks = plan.predicted.partition_scoped_checks;
        let retained_count = summary.counters.frontier_trace_retained_count;
        graph.with_telemetry(|telemetry| {
            telemetry.invalidation.partition_scoped_invalidation_checks += partition_checks;
            telemetry.invalidation.frontier_trace_retained_count += retained_count;
        });
        graph.record_frontier_execution_diagnostics(plan.predicted.clone(), summary, trace_records);
        Ok(())
    })();

    if let Err(err) = result {
        graph.clear_pending_diagnostics_input();
        if graph.captures_failure_diagnostics() {
            DiagnosticsRecorder::new(graph).record_failure(ExecutionFailureContext::from_error(
                ExecutionFailurePhase::Invalidation,
                &err,
                None,
            ));
        }
        return Err(err);
    }
    Ok(admission)
}

fn plan_invalidation_frontier(
    graph: &mut SignalGraph,
    dirty: &DirtyBatch,
) -> Result<FrontierPlan, SignalError> {
    planning::plan_invalidation_frontier(graph, dirty)
}

fn execute_invalidation_frontier(
    graph: &mut SignalGraph,
    plan: &FrontierPlan,
    capture_frontier: bool,
) -> Result<Option<crate::data::proof::FrontierDiagnosticsSidecar>, SignalError> {
    application::execute_invalidation_frontier(graph, plan, capture_frontier)
}

fn retained_trace_records(
    graph: &SignalGraph,
    plan: &FrontierPlan,
) -> Result<Vec<InvalidationTraceRecord>, SignalError> {
    evidence::retained_trace_records(graph, plan)
}

fn mark_source_seed(graph: &mut SignalGraph, seed: &InvalidationSeed) -> Result<(), SignalError> {
    let source_was_clean = matches!(graph.get_state(seed.source_node)?, NodeState::Clean);
    graph.transition_node_dirty(
        seed.source_node,
        seed.aspect,
        seed.changed_scopes.as_slice(),
    )?;
    if source_was_clean {
        record_invalidation_lineage(
            graph,
            seed.source_node,
            InvalidationCause::SourceAspectChanged {
                aspect_index: seed.aspect.index(),
            },
        )?;
    }
    Ok(())
}
