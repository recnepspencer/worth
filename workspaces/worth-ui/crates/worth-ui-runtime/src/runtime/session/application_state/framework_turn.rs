use super::WorthUiApplicationSessionState;
use crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity;
use crate::facade::registry::snapshot::CapabilitySnapshot;
use crate::graph::UiGraphAuthority;
use crate::runtime::{WorthUiFrameworkTurn, WorthUiFrameworkTurnCompletion};

pub(crate) struct WorthUiApplicationFrameworkTurnCompletion<'session> {
    generation_identity: WorthUiPreparedApplicationGenerationIdentity,
    visual_trace_source:
        crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource,
    graph: UiGraphAuthority<'session>,
    active_plan_digest: u64,
    completion: WorthUiFrameworkTurnCompletion<'session>,
    capabilities: &'session CapabilitySnapshot,
    intent_catalog: &'session crate::declaration::UiIntentCatalog,
    consumed_facts: &'session crate::graph::UiGraphConsumedFactIndex,
}

impl WorthUiApplicationSessionState {
    pub(crate) fn execute_framework_turn(
        &mut self,
        collect_sources: impl FnOnce(&mut WorthUiFrameworkTurn<'_>),
    ) -> WorthUiApplicationFrameworkTurnCompletion<'_> {
        let generation_identity = self.app.generation_identity().clone();
        let visual_trace_source = self.app.visual_trace_source();
        let graph = self.app.graph();
        let capabilities = self.app.capabilities();
        let intent_catalog = self.app.prepared_authority().intent_catalog();
        let consumed_facts = self.app.prepared_authority().consumed_fact_index();
        let active_plan_digest = self.runtime.active.active_plan_ref().digest().as_u64();
        let completion = self.runtime.execute_framework_turn(collect_sources);
        WorthUiApplicationFrameworkTurnCompletion {
            generation_identity,
            visual_trace_source,
            graph,
            active_plan_digest,
            completion,
            capabilities,
            intent_catalog,
            consumed_facts,
        }
    }

    pub(crate) fn prepare_empty_activation_boundary(
        &mut self,
    ) -> Result<crate::runtime::WorthUiFrameBoundary, ()> {
        self.runtime
            .execute_framework_turn(|_| {})
            .into_execution()
            .map(|execution| execution.into_activation_boundary())
            .map_err(|_| ())
    }
}

impl<'session> WorthUiApplicationFrameworkTurnCompletion<'session> {
    pub(crate) fn into_parts(
        self,
    ) -> (
        WorthUiPreparedApplicationGenerationIdentity,
        crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource,
        UiGraphAuthority<'session>,
        u64,
        WorthUiFrameworkTurnCompletion<'session>,
        &'session CapabilitySnapshot,
        &'session crate::declaration::UiIntentCatalog,
        &'session crate::graph::UiGraphConsumedFactIndex,
    ) {
        (
            self.generation_identity,
            self.visual_trace_source,
            self.graph,
            self.active_plan_digest,
            self.completion,
            self.capabilities,
            self.intent_catalog,
            self.consumed_facts,
        )
    }
}
