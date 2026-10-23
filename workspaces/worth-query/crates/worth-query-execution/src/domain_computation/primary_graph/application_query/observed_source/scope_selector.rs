use std::marker::PhantomData;
use std::sync::Arc;

use super::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector;

impl<Query> WorthQueryObservedSource<Query> {
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
