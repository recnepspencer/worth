//! Inspection's original association and scope contain no business input or snapshot.
use super::OriginalSourceDisclosure;
use crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector;
use std::sync::Arc;
use worth_query_installation::facade::InstalledSourceDisclosureBinding;

pub(in crate::domain_computation) struct OriginalSourceInspectionDisclosure {
    association: Arc<InstalledSourceDisclosureBinding>,
    scope: Arc<WorthQueryObservedScopeSelector>,
    disclosure: OriginalSourceDisclosure,
}

impl OriginalSourceInspectionDisclosure {
    pub(super) fn captured(
        association: Arc<InstalledSourceDisclosureBinding>,
        scope: Arc<WorthQueryObservedScopeSelector>,
        disclosure: OriginalSourceDisclosure,
    ) -> Self {
        Self {
            association,
            scope,
            disclosure,
        }
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
        self.association == other.association
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
