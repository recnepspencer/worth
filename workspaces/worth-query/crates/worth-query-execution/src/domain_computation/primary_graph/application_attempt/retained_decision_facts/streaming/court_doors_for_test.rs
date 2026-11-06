//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial;

impl AdmittedFactKey {
    #[doc(hidden)]
    pub fn from_borrowed_for_test(
        locator: impl Fn(&mut dyn fmt::Write) -> fmt::Result,
        predicate: Option<(&AspectValue, usize)>,
        policy: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::from_borrowed(locator, predicate, policy)
    }
}
