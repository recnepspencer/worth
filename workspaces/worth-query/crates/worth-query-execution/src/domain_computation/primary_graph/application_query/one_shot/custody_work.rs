//! Copy work for the observed-source custody a one-shot result retains.

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::history::BranchId;

use super::super::observed_source::WorthQueryObservedScopeSelector;

/// The copies a one-shot result performs to retain its observed-source
/// custody: the descriptive scope selector, then the shared query-name and
/// branch descriptors. They scale with those descriptors, not with anything the
/// read examined, so they are never charged against the Query's declared work
/// limit. Their bytes are claimed from the result buffer; an admitted read also
/// pays this work on its carried admission.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph::application_query) struct RetainedCustodyWork {
    pub(in crate::domain_computation::primary_graph::application_query) scope: usize,
    pub(in crate::domain_computation::primary_graph::application_query) descriptor: usize,
}

impl RetainedCustodyWork {
    pub(in crate::domain_computation::primary_graph::application_query) fn of(
        scope_locator: &AspectFieldLocator,
        scope_value: &AspectValue,
        query_name: &str,
        branch: &BranchId,
    ) -> Option<Self> {
        Some(Self {
            scope: WorthQueryObservedScopeSelector::copy_work(scope_locator, scope_value)?,
            descriptor: query_name.len().checked_add(branch.0.len())?,
        })
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn total(
        self,
    ) -> Option<usize> {
        self.scope.checked_add(self.descriptor)
    }
}
