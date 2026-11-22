//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryObservedPreImageCandidate {
    #[doc(hidden)]
    pub fn from_observed_field_for_test(
        read_scope: WorthQueryOperationGraphReadScope,
        locator: AspectFieldLocator,
        value: AspectValue,
        entity_id: EntityId,
        entity_kind: KindId,
    ) -> Option<Self> {
        Self::from_observed_field(read_scope, locator, value, entity_id, entity_kind)
    }
}

impl WorthQueryRetainedPreImageSelection {
    #[doc(hidden)]
    pub fn into_parts_for_test(self) -> (WorthQueryRetainedPreImage, usize) {
        Self::into_parts(self)
    }
}

impl WorthQueryObservedPreImageCandidate {
    #[doc(hidden)]
    pub fn locator_for_test(&self) -> &AspectFieldLocator {
        Self::locator(self)
    }
}

impl WorthQueryObservedPreImageCandidate {
    #[doc(hidden)]
    pub fn target_record_for_test(&self) -> &RecordRef {
        Self::target_record(self)
    }
}
