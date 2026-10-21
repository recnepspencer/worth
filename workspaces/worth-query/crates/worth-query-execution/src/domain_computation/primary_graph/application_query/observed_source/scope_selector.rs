use std::marker::PhantomData;
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};

use super::WorthQueryObservedSource;

/// The original descriptive scope of a one-shot query. A later request must
/// resolve it again; its previously selected entity ID grants no read access.
pub(in crate::domain_computation::primary_graph) struct WorthQueryObservedScopeSelector {
    locator: AspectFieldLocator,
    value: AspectValue,
}

impl WorthQueryObservedScopeSelector {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        locator: &AspectFieldLocator,
        value: &AspectValue,
    ) -> Self {
        Self {
            locator: locator.clone(),
            value: value.clone(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn locator(&self) -> &AspectFieldLocator {
        &self.locator
    }

    pub(in crate::domain_computation::primary_graph) fn value(&self) -> &AspectValue {
        &self.value
    }
}

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
