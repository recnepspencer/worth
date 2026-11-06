//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial;

impl AdmittedFactKey {
    #[doc(hidden)]
    pub fn locator_for_test(&self) -> &[u8] {
        Self::locator(self)
    }
}

impl AdmittedFactKey {
    #[doc(hidden)]
    pub fn write_for_test(
        locator_length: usize,
        predicate: Option<(usize, usize)>,
        policy: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
        locator: impl FnOnce(&mut ExecutionByteBuffer) -> Result<(), StoreDenial>,
        predicate_writer: impl FnOnce(&mut ExecutionByteBuffer) -> Result<(), StoreDenial>,
    ) -> Result<Self, StoreDenial> {
        Self::write(locator_length, predicate, policy, locator, predicate_writer)
    }
}
