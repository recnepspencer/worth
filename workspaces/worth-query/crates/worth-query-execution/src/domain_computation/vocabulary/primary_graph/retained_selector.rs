//! Moved whole from `domain_computation::primary_graph::entity_resolution::retained_selector`: the items below and every impl they own.
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::EntityId;

pub struct WorthQueryObservedScopeSelector {
    pub(in crate::domain_computation) locator: AspectFieldLocator,
    pub(in crate::domain_computation) value: AspectValue,
    pub(in crate::domain_computation) subject: EntityId,
    // Last to drop: the original request remains charged through the final share.
    pub(in crate::domain_computation) _funding: Option<worth_execution::ExecutionMemoryReservation>,
}

impl WorthQueryObservedScopeSelector {
    pub(in crate::domain_computation) fn locator(&self) -> &AspectFieldLocator {
        &self.locator
    }
    pub(in crate::domain_computation) fn value(&self) -> &AspectValue {
        &self.value
    }
    pub(in crate::domain_computation) fn subject(&self) -> EntityId {
        self.subject
    }
}

impl WorthQueryObservedScopeSelector {
    #[doc(hidden)]
    pub fn subject_for_test(&self) -> EntityId {
        Self::subject(self)
    }
}
