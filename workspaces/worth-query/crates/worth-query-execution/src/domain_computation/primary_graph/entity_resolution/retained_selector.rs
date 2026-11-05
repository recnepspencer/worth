//! Descriptive scope retained from native entity resolution, never read authority.
use super::WorthQueryApplicationEntityIdentity;
use crate::domain_computation::vocabulary::primary_graph::retained_selector::WorthQueryObservedScopeSelector;
use std::sync::Arc;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
mod funding;

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

mod court_doors_for_test;
