//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial;

impl<T> RunDirectory<T> {
    #[doc(hidden)]
    pub fn ensure_for_test(
        &mut self,
        level: usize,
        policy: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
    ) -> Result<(), StoreDenial> {
        Self::ensure(self, level, policy)
    }
}
