//! Descriptive scope retained from native entity resolution, never read authority.
use super::WorthQueryApplicationEntityIdentity;
use std::sync::Arc;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::EntityId;
mod funding;

pub(in crate::domain_computation) struct WorthQueryObservedScopeSelector {
    locator: AspectFieldLocator,
    value: AspectValue,
    subject: EntityId,
    // Last to drop: the original request remains charged through the final share.
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}

impl WorthQueryObservedScopeSelector {
    pub(in crate::domain_computation::primary_graph) fn locator(&self) -> &AspectFieldLocator {
        &self.locator
    }
    pub(in crate::domain_computation::primary_graph) fn value(&self) -> &AspectValue {
        &self.value
    }
    pub(in crate::domain_computation) fn subject(&self) -> EntityId {
        self.subject
    }
}

impl<Schema, Entity> WorthQueryApplicationEntityIdentity<Schema, Entity> {
    pub(in crate::domain_computation) fn retain_scope_selector(
        &self,
    ) -> Arc<WorthQueryObservedScopeSelector> {
        Arc::new(WorthQueryObservedScopeSelector {
            locator: self.identity_locator().clone(),
            value: self.identity_value().clone(),
            subject: self.entity_id(),
            _funding: None,
        })
    }
}
