//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial;

impl<T> RetainedFactStore<T> {
    #[doc(hidden)]
    pub fn finish_for_test(
        self,
        policy: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
    ) -> Result<ExecutionArray<RetainedFact<T>>, StoreDenial> {
        Self::finish(self, policy)
    }
}

impl<T> RetainedFactStore<T> {
    #[doc(hidden)]
    pub fn from_sorted_records_for_test(
        records: ExecutionArray<RetainedFact<T>>,
        policy: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::from_sorted_records(records, policy)
    }
}

impl<T> RetainedFactStore<T> {
    #[doc(hidden)]
    pub fn insert_for_test(
        &mut self,
        key: AdmittedFactKey,
        value: T,
        policy: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
        duplicate: impl FnOnce(&mut T, T) -> Result<(), StoreDenial>,
    ) -> Result<(), StoreDenial> {
        Self::insert(self, key, value, policy, duplicate)
    }
}
