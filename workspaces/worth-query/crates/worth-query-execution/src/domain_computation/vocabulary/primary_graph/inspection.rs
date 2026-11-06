//! Moved whole from `domain_computation::primary_graph::source_disclosure::inspection`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::original::OriginalSourceDisclosure;
use crate::domain_computation::vocabulary::primary_graph::retained_selector::WorthQueryObservedScopeSelector;
use std::sync::Arc;
use worth_query_installation::facade::InstalledSourceDisclosureBinding;

pub struct OriginalSourceInspectionDisclosure {
    schema: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    association: Arc<InstalledSourceDisclosureBinding>,
    scope: Arc<WorthQueryObservedScopeSelector>,
    disclosure: OriginalSourceDisclosure,
}

impl OriginalSourceInspectionDisclosure {
    pub(in crate::domain_computation) fn captured(
        schema: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        association: Arc<InstalledSourceDisclosureBinding>,
        scope: Arc<WorthQueryObservedScopeSelector>,
        disclosure: OriginalSourceDisclosure,
    ) -> Self {
        Self {
            schema,
            association,
            scope,
            disclosure,
        }
    }
    pub(in crate::domain_computation) fn schema(
        &self,
    ) -> &worth_query_installation::facade::ApplicationSchemaBindingIdentity {
        &self.schema
    }
    pub(in crate::domain_computation) fn association(&self) -> &InstalledSourceDisclosureBinding {
        &self.association
    }
    pub(in crate::domain_computation) fn scope(&self) -> &WorthQueryObservedScopeSelector {
        &self.scope
    }
    pub(in crate::domain_computation) fn disclosure(&self) -> &OriginalSourceDisclosure {
        &self.disclosure
    }
}

impl PartialEq for OriginalSourceInspectionDisclosure {
    fn eq(&self, other: &Self) -> bool {
        self.schema == other.schema
            && self.association == other.association
            && Arc::ptr_eq(&self.scope, &other.scope)
            && self.disclosure.same_capture(&other.disclosure)
    }
}

impl Eq for OriginalSourceInspectionDisclosure {}

impl std::fmt::Debug for OriginalSourceInspectionDisclosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginalSourceInspectionDisclosure")
            .finish_non_exhaustive()
    }
}

impl OriginalSourceInspectionDisclosure {
    #[doc(hidden)]
    pub fn scope_for_test(&self) -> &WorthQueryObservedScopeSelector {
        Self::scope(self)
    }
}

mod court_doors_for_test;
