use worth_proof::TransitionOutcome;

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::progression::{
    InvalidationProgressionOwner, ReadyInvalidationBatch,
};
use crate::data::telemetry::InvalidationPerformedCounter;

pub(crate) fn execute_ready<Outcome>(
    graph: &SignalGraph,
    ready: ReadyInvalidationBatch,
    effect: impl FnOnce() -> Result<Outcome, SignalError>,
) -> Result<Outcome, SignalError> {
    execute_ready_with_work(
        graph,
        ready,
        &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        effect,
    )
}

pub(crate) fn execute_ready_with_work<Outcome>(
    graph: &SignalGraph,
    ready: ReadyInvalidationBatch,
    work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    effect: impl FnOnce() -> Result<Outcome, SignalError>,
) -> Result<Outcome, SignalError> {
    if let Err(error) = super::readiness::ensure_ready_is_current(graph, &ready) {
        graph
            .invalidation_performed_counter_state()
            .add(InvalidationPerformedCounter::StaleWorkRejected, 1);
        return Err(error);
    }
    let observation_generation = graph.observation_session_active_generation();
    let capture = graph.prepare_invalidation_performed_work(
        InvalidationProgressionOwner::ready_binding(&ready),
        work,
    )?;
    match InvalidationProgressionOwner::execute(ready, |_| effect()) {
        TransitionOutcome::Success(executed) => {
            // The effect belongs to the observation present at admission. A
            // cancelled/superseded session cannot donate it to another scope.
            if graph.observation_session_active_generation() == observation_generation {
                if let Some(capture) = capture {
                    capture.commit();
                }
                graph.record_observation_execution_boundary();
                graph
                    .invalidation_performed_counter_state()
                    .add(InvalidationPerformedCounter::NodesEvaluated, 1);
            }
            Ok(InvalidationProgressionOwner::into_executed_outcome(
                executed,
            ))
        }
        TransitionOutcome::Failed(error) => Err(error),
        TransitionOutcome::Denied(never)
        | TransitionOutcome::Deferred(never)
        | TransitionOutcome::Stale(never)
        | TransitionOutcome::RebindRequired(never) => match never {},
    }
}
