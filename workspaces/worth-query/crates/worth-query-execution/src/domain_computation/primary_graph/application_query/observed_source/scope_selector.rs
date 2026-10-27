use std::marker::PhantomData;
use std::sync::Arc;

use super::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector;

impl<Query> WorthQueryObservedSource<Query> {
    /// A genuine settlement seed, not proof of row or field production.
    /// Metadata must be freshly admitted before inspecting its availability.
    pub(in crate::domain_computation::primary_graph) fn original_settled_producer(
        &self,
    ) -> Option<&super::super::super::application_output_demand::OriginalSettledProducerObservation>
    {
        self.parameters.producer_observation()
    }

    /// Original completed rows, distinct from the result-set selection meaning.
    /// Descriptive custody only; callers must admit metadata before traversal.
    pub(in crate::domain_computation::primary_graph) fn original_row_footprints(
        &self,
    ) -> impl ExactSizeIterator<Item = &super::WorthQueryObservedSourceFootprint> {
        self.parameters.row_footprints()
    }

    pub(in crate::domain_computation::primary_graph) fn original_binding(
        &self,
    ) -> Option<
        &worth_query_installation::facade::WorthQueryInstalledApplicationQueryBindingObservation,
    > {
        self.parameters.binding_observation()
    }

    pub(in crate::domain_computation::primary_graph) fn retained_scope_selector(
        &self,
    ) -> &WorthQueryObservedScopeSelector {
        &self.scope_selector
    }

    /// The same source under another query marker. Only a caller that has
    /// already checked the query this source names may change its marker.
    pub(in crate::domain_computation::primary_graph) fn retyped<Other>(
        &self,
    ) -> WorthQueryObservedSource<Other> {
        WorthQueryObservedSource {
            expression_context: self.expression_context.clone(),
            runtime_authority: self.runtime_authority,
            schema_binding: self.schema_binding.clone(),
            query_identity: self.query_identity.clone(),
            parameter_binding_identity: self.parameter_binding_identity,
            parameters: Arc::clone(&self.parameters),
            query_identifier: self.query_identifier.clone(),
            branch: self.branch.clone(),
            selection: self.selection.clone(),
            model_root: self.model_root,
            source_meaning: Arc::clone(&self.source_meaning),
            scope_selector: Arc::clone(&self.scope_selector),

            _marker: PhantomData,
        }
    }
}
