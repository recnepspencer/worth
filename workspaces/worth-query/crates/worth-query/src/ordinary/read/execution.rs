use super::{
    admit_read_context_declaration, WorthQueryReadCompletion, WorthQueryReadJourneyCounters,
    WorthQueryReadOutcome, WorthQueryReadProjectionBinding, WorthQueryReadRequest,
    WorthQueryReadStop,
};
use crate::runtime::WorthQueryWorkspace;

impl WorthQueryReadRequest {
    /// Admit the declared authority context and execute the read through
    /// Query-owned planning, routing, and receipt assembly.
    pub fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryReadOutcome {
        let (declaration, context) = self.into_parts();
        let intent = declaration.into_declared_intent();
        let journey_counters = WorthQueryReadJourneyCounters::begin_context_admission();
        let admitted_context = match admit_read_context_declaration(&intent, context) {
            Ok(context) => context,
            Err(denial) => {
                return WorthQueryReadOutcome::Stopped(WorthQueryReadStop::context(
                    denial,
                    journey_counters,
                ));
            }
        };
        let (authority, planning_authority, context_receipt) = admitted_context.into_parts();
        let journey_counters = journey_counters.record_planning_attempt();
        let read_graph = match intent.plan(planning_authority) {
            Ok(read_graph) => read_graph,
            Err(denial) => {
                return WorthQueryReadOutcome::Stopped(WorthQueryReadStop::planning(
                    denial,
                    context_receipt,
                    journey_counters,
                ));
            }
        };
        let journey_counters = journey_counters.record_planning_completed();
        let projection_binding = WorthQueryReadProjectionBinding::from_graph(&read_graph);
        let journey_counters = journey_counters.record_lower_runtime_execution_attempt();
        let runtime_result =
            workspace.execute_declared_read_graph_in_authority(execution, read_graph, &authority);
        match runtime_result {
            Ok(result) => WorthQueryReadOutcome::Completed(WorthQueryReadCompletion::new(
                result,
                context_receipt,
                journey_counters.record_lower_runtime_execution_completed(),
                projection_binding,
                workspace.runtime_authority_identity(),
            )),
            Err(error) => WorthQueryReadOutcome::Stopped(WorthQueryReadStop::runtime(
                error,
                context_receipt,
                journey_counters,
            )),
        }
    }
}
