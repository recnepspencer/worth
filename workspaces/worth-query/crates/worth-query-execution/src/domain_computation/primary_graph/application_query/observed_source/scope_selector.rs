use std::marker::PhantomData;
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};

use super::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryRetainedSourceCharge;

/// The original descriptive scope of a one-shot query. A later request must
/// resolve it again; its previously selected entity ID grants no read access.
pub(in crate::domain_computation::primary_graph) struct WorthQueryObservedScopeSelector {
    locator: AspectFieldLocator,
    value: AspectValue,
    _charge: WorthQueryRetainedSourceCharge,
}

impl WorthQueryObservedScopeSelector {
    pub(in crate::domain_computation::primary_graph::application_query) fn charged_bytes(
        locator: &AspectFieldLocator,
        value: &AspectValue,
    ) -> Option<usize> {
        std::mem::size_of::<(usize, usize, Self)>()
            .checked_add(locator.owned_allocation_capacity_bytes())?
            .checked_add(value.owned_allocation_capacity_bytes())
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn copy_work(
        locator: &AspectFieldLocator,
        value: &AspectValue,
    ) -> Option<usize> {
        let field_path = locator.field_path();
        // Work counts the initialized fields visited and the bytes copied by
        // owned strings. Inline wrapper widths belong to retained capacity,
        // not to the number of performed copy steps.
        let locator_work = field_path.fields().iter().try_fold(
            locator.aspect().aspect_key().as_str().len(),
            |work, field| work.checked_add(1)?.checked_add(field.as_str().len()),
        )?;
        let value_heap_work = match value {
            AspectValue::Decimal(value) => value.as_str().len(),
            AspectValue::BigInt(value) => value.as_str().len(),
            AspectValue::Rational(value) => value
                .numerator
                .as_str()
                .len()
                .checked_add(value.denominator.as_str().len())?,
            AspectValue::String(InternedString::Raw(value)) => value.len(),
            _ => 0,
        };
        2_usize
            .checked_add(locator_work)?
            .checked_add(value_heap_work)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        locator: &AspectFieldLocator,
        value: &AspectValue,
        charge: WorthQueryRetainedSourceCharge,
    ) -> Self {
        Self {
            locator: locator.clone(),
            value: value.clone(),
            _charge: charge,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn locator(&self) -> &AspectFieldLocator {
        &self.locator
    }

    pub(in crate::domain_computation::primary_graph) fn value(&self) -> &AspectValue {
        &self.value
    }

    pub(in crate::domain_computation::primary_graph) fn initialized_copy_work(
        &self,
    ) -> Option<usize> {
        Self::copy_work(&self.locator, &self.value)
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
            _descriptor_charge: Arc::clone(&self._descriptor_charge),
            _marker: PhantomData,
        }
    }
}
