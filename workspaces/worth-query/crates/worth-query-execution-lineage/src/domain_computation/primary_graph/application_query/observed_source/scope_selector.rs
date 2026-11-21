use std::sync::Arc;

use super::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector;

impl<Query> WorthQueryObservedSource<Query> {
    /// Original descriptive projection custody. Fresh inspection admission
    /// must precede checking availability or traversing its contents.
    pub fn original_projection(
        &self,
    ) -> Option<&Arc<super::super::projection::retained_projection::OriginalProjection>> {
        self.parameters.projection()
    }

    /// A genuine settlement seed, not proof of row or field production.
    /// Metadata must be freshly admitted before inspecting its availability.
    pub fn original_settled_producer(
        &self,
    ) -> Option<&super::super::super::application_output_demand::OriginalSettledProducerObservation>
    {
        self.parameters.producer_observation()
    }

    /// Original completed rows, distinct from the result-set selection meaning.
    /// Descriptive custody only; callers must admit metadata before traversal.
    pub fn original_row_footprints(
        &self,
    ) -> impl ExactSizeIterator<Item = &super::WorthQueryObservedSourceFootprint> {
        self.parameters.row_footprints()
    }

    pub fn original_binding(
        &self,
    ) -> Option<
        &worth_query_installation::facade::WorthQueryInstalledApplicationQueryBindingObservation,
    > {
        self.parameters.binding_observation()
    }

    pub fn retained_scope_selector(&self) -> &Arc<WorthQueryObservedScopeSelector> {
        &self.scope_selector
    }

    /// Exact original immutable custody, rather than public receipt equality.
    pub fn shares_original_custody<Other>(&self, other: &WorthQueryObservedSource<Other>) -> bool {
        (match (
            &self.invocation_managed_source,
            &other.invocation_managed_source,
        ) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }) && Arc::ptr_eq(&self.parameters, &other.parameters)
            && Arc::ptr_eq(&self.source_meaning, &other.source_meaning)
            && self.runtime_authority == other.runtime_authority
            && self.schema_binding == other.schema_binding
            && self.query_identity == other.query_identity
            && self.parameter_binding_identity == other.parameter_binding_identity
            && self.selection == other.selection
            && self.model_root == other.model_root
    }
}

mod court_doors_for_test;
