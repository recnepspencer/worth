//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial;

impl<T> Run<T> {
    #[doc(hidden)]
    pub fn slots_for_test(&self) -> &Slots<T> {
        &self.slots
    }
}

impl<T> Run<T> {
    #[doc(hidden)]
    pub fn used_mut_for_test(&mut self) -> &mut usize {
        &mut self.used
    }
}

impl<T> Run<T> {
    #[doc(hidden)]
    pub fn locate_for_test(
        &self,
        key: &AdmittedFactKey,
        policy: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
    ) -> Result<Option<&Slot<T>>, StoreDenial> {
        Self::locate(self, key, policy)
    }
}

impl<T> Run<T> {
    #[doc(hidden)]
    pub fn merge_for_test(
        left: Self,
        right: Self,
        policy: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::merge(left, right, policy)
    }
}
