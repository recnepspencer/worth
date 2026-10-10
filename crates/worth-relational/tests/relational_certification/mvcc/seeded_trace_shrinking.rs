use super::{replay_trace, ProductionModelTrace, ScenarioLifecycle, TraceFailure};

pub(super) fn shrink_failing_trace(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut trace: ProductionModelTrace,
    mut failure: TraceFailure,
) -> (ProductionModelTrace, TraceFailure) {
    let mut scenario_index = 0;
    while scenario_index < trace.scenarios.len() {
        let mut candidate = trace.clone();
        candidate.scenarios.remove(scenario_index);
        if candidate.scenarios.len() < 2 {
            scenario_index += 1;
            continue;
        }
        normalize_lifecycles(&mut candidate);
        match replay_trace(execution, &candidate) {
            Err(candidate_failure) if candidate_failure.identity == failure.identity => {
                trace = candidate;
                failure = candidate_failure;
            }
            _ => scenario_index += 1,
        }
    }
    (trace, failure)
}

fn normalize_lifecycles(trace: &mut ProductionModelTrace) {
    let last = trace.scenarios.len() - 1;
    for (index, scenario) in trace.scenarios.iter_mut().enumerate() {
        scenario.lifecycle = if index == 0 {
            ScenarioLifecycle::RetainArchiveObserveRelease
        } else if index == last {
            ScenarioLifecycle::DeleteAfterCommit
        } else {
            ScenarioLifecycle::CommitOnly
        };
    }
}
