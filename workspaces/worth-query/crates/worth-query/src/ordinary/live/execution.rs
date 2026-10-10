use crate::ordinary::read::{
    admit_read_context_declaration, WorthQueryReadJourneyCounters, WorthQueryReadProjectionBinding,
    WorthQueryReadStop,
};
use crate::runtime::WorthQueryWorkspace;

use super::{
    WorthQueryLiveOpenCompletion, WorthQueryLiveOpenOutcome, WorthQueryLiveOpenStop,
    WorthQueryLiveRequest, WorthQueryManagedLiveHandle,
};

impl WorthQueryLiveRequest {
    pub fn open(
        self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryLiveOpenOutcome {
        let (declaration, context) = self.into_parts();
        let (name, read) = declaration.into_parts();
        let intent = read.into_declared_intent();
        let journey = WorthQueryReadJourneyCounters::begin_context_admission();
        let admitted_context = match admit_read_context_declaration(&intent, context) {
            Ok(context) => context,
            Err(denial) => {
                return stopped(WorthQueryReadStop::context(denial, journey));
            }
        };
        let (authority, planning_authority, context_receipt) = admitted_context.into_parts();
        let journey = journey.record_planning_attempt();
        let read_graph = match intent.plan(planning_authority) {
            Ok(read_graph) => read_graph,
            Err(denial) => {
                return stopped(WorthQueryReadStop::planning(
                    denial,
                    context_receipt,
                    journey,
                ));
            }
        };
        let journey = journey
            .record_planning_completed()
            .record_lower_runtime_execution_attempt();
        let projection_binding = WorthQueryReadProjectionBinding::from_graph(&read_graph);
        match workspace.open_declared_live_graph(request_execution, name, read_graph, &authority) {
            Ok(view) => WorthQueryLiveOpenOutcome::Opened(WorthQueryLiveOpenCompletion::new(
                WorthQueryManagedLiveHandle::new(
                    view,
                    workspace.managed_live_capability(),
                    projection_binding,
                ),
                context_receipt,
                journey.record_lower_runtime_execution_completed(),
            )),
            Err(error) => stopped(WorthQueryReadStop::runtime(error, context_receipt, journey)),
        }
    }
}

fn stopped(stop: WorthQueryReadStop) -> WorthQueryLiveOpenOutcome {
    WorthQueryLiveOpenOutcome::Stopped(WorthQueryLiveOpenStop::new(stop))
}
