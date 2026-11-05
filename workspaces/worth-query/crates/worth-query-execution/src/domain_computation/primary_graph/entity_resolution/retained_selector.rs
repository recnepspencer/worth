//! Descriptive scope retained from native entity resolution, never read authority.
use super::{WorthQueryApplicationEntityIdentity, WorthQueryResolvedEntity};
use std::sync::Arc;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::EntityId;
mod funding;

pub struct WorthQueryObservedScopeSelector {
    description: WorthQueryResolvedEntity,
    // Last to drop: the original request remains charged through the final share.
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}

impl WorthQueryObservedScopeSelector {
    /// The original owner-sealed description is sufficient to request current
    /// authorization; it proves neither presence nor permission at a new basis.
    fn describe<Schema, Entity>(&self) -> WorthQueryApplicationEntityIdentity<Schema, Entity> {
        self.description.clone().into_application_identity()
    }

    pub(in crate::domain_computation::primary_graph) fn matches_source<Q>(
        &self,
        source: &super::super::WorthQueryObservedSource<Q>,
    ) -> bool {
        self.description.matches_source(source)
    }
    pub(in crate::domain_computation::primary_graph) fn locator(&self) -> &AspectFieldLocator {
        self.description.locator()
    }
    pub(in crate::domain_computation::primary_graph) fn value(&self) -> &AspectValue {
        self.description.value()
    }
    pub(in crate::domain_computation) fn subject(&self) -> EntityId {
        self.description.entity_id()
    }
}

impl<Schema, Entity> WorthQueryApplicationEntityIdentity<Schema, Entity> {
    pub(in crate::domain_computation) fn retain_scope_selector(
        &self,
    ) -> Arc<WorthQueryObservedScopeSelector> {
        Arc::new(WorthQueryObservedScopeSelector {
            description: self.retain_description(),
            _funding: None,
        })
    }
}

mod court_doors_for_test;
