use crate::ordinary::read::{
    admit_read_context_declaration, WorthQueryReadJourneyCounters, WorthQueryReadStop,
};
use crate::runtime::WorthQueryWorkspace;

use super::{WorthQueryCountCompletion, WorthQueryCountOutcome, WorthQueryCountRequest};

impl WorthQueryCountRequest {
    pub fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryCountOutcome {
        let (declaration, context) = self.into_parts();
        let intent = declaration.into_declared_intent();
        let journey_counters = WorthQueryReadJourneyCounters::begin_context_admission();
        let admitted_context = match admit_read_context_declaration(&intent, context) {
            Ok(context) => context,
            Err(denial) => {
                return WorthQueryCountOutcome::Stopped(WorthQueryReadStop::context(
                    denial,
                    journey_counters,
                ));
            }
        };
        let (authority, planning_authority, context_receipt) = admitted_context.into_parts();
        let journey_counters = journey_counters.record_planning_attempt();
        let read_graph = match intent.plan_count(planning_authority) {
            Ok(read_graph) => read_graph,
            Err(denial) => {
                return WorthQueryCountOutcome::Stopped(WorthQueryReadStop::planning(
                    denial,
                    context_receipt,
                    journey_counters,
                ));
            }
        };
        let journey_counters = journey_counters.record_planning_completed();
        let journey_counters = journey_counters.record_lower_runtime_execution_attempt();
        match workspace.execute_declared_count_graph_in_authority(execution, read_graph, &authority)
        {
            Ok(result) => WorthQueryCountOutcome::Completed(WorthQueryCountCompletion::new(
                result,
                context_receipt,
                journey_counters.record_lower_runtime_execution_completed(),
            )),
            Err(error) => WorthQueryCountOutcome::Stopped(WorthQueryReadStop::runtime(
                error,
                context_receipt,
                journey_counters,
            )),
        }
    }
}
