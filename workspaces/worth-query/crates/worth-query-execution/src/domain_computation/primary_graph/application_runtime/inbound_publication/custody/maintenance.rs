//! Bounded operation-scoped selection of retained transport completions.

use std::ops::Bound::{Excluded, Unbounded};

use worth_query_installation::facade::ApplicationSchema;

use super::InstalledTransportCompletionCustody;
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl InstalledTransportCompletionCustody {
    fn candidates(
        &mut self,
        operation: &str,
        maximum_work: usize,
    ) -> (Vec<ExternalEffectCorrelationIdentity>, usize) {
        let Some(pending) = self.pending_by_operation.get(operation) else {
            return (Vec::new(), 0);
        };
        let available = pending.len();
        let mut selected = Vec::with_capacity(maximum_work.min(available));
        for _ in 0..maximum_work.min(available) {
            let next = self
                .maintenance_cursor_by_operation
                .get(operation)
                .copied()
                .and_then(|cursor| pending.range((Excluded(cursor), Unbounded)).next().copied())
                .or_else(|| pending.first().copied());
            let Some(correlation) = next else { break };
            self.maintenance_cursor_by_operation
                .insert(operation.to_owned(), correlation);
            selected.push(correlation);
        }
        (selected, available)
    }

    fn pending_count(&self, operation: &str) -> usize {
        self.pending_by_operation
            .get(operation)
            .map_or(0, |pending| pending.len())
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Select one finite batch without scanning another installed operation.
    pub(in crate::domain_computation::primary_graph) fn transport_maintenance_candidates(
        &self,
        operation: &str,
        maximum_work: usize,
    ) -> (Vec<ExternalEffectCorrelationIdentity>, usize) {
        self.transport_completion_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .candidates(operation, maximum_work)
    }

    pub(in crate::domain_computation::primary_graph) fn pending_transport_maintenance_count(
        &self,
        operation: &str,
    ) -> usize {
        self.transport_completion_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pending_count(operation)
    }
}
