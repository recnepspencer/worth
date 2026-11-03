//! Captured input summaries share the existing bounded progress retention owner.
use super::*;
use crate::domain_computation::primary_graph::workflow::captured_input::WorkflowCapturedInputSupport;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RetainedCapturedInputs {
    transition: WorkflowTransitionLocator,
    support: Arc<WorkflowCapturedInputSupport>,
}
impl WorkflowInstanceProgress {
    pub(in crate::domain_computation::primary_graph) fn captured_input_support(
        &self,
        source: WorkflowTransitionLocator,
    ) -> Option<&WorkflowCapturedInputSupport> {
        self.captured_inputs
            .as_ref()?
            .get(&source.settlement().node())
            .filter(|retained| retained.transition == source)
            .map(|retained| retained.support.as_ref())
    }
    pub(super) fn retain_captured_inputs(
        &mut self,
        source: WorkflowTransitionLocator,
        support: Arc<WorkflowCapturedInputSupport>,
    ) {
        let entries = self.captured_inputs.get_or_insert_with(OrdMap::new);
        let node = source.settlement().node();
        if entries.get(&node).is_some_and(|retained| {
            retained.transition.settlement().occurrence() >= source.settlement().occurrence()
        }) {
            return;
        }
        entries.insert(
            node,
            RetainedCapturedInputs {
                transition: source,
                support,
            },
        );
    }
    pub(super) fn captured_input_charge_bytes(&self) -> usize {
        self.captured_inputs.as_ref().map_or(0, |entries| {
            entries
                .values()
                .fold(accounting::map_charge_bytes(entries), |bytes, retained| {
                    bytes.saturating_add(retained.support.retained_charge_bytes())
                })
        })
    }
}
