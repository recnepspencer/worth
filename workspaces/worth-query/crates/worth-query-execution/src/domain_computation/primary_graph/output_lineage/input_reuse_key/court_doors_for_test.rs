//! Sealed test doors over this module's private members.
use super::*;

impl PreparedInputReuseKey {
    #[doc(hidden)]
    pub fn new_for_test(
        selection: WorthQueryObservedSourceSelection,
        input_identity: [u8; 32],
        installed_edition: InstalledProducerEdition,
    ) -> Self {
        Self::new(selection, input_identity, installed_edition)
    }
}

impl PreparedInputReuseKey {
    #[doc(hidden)]
    pub fn continued_by_republication_for_test(&self) -> Self {
        Self::continued_by_republication(self)
    }
}

impl PreparedInputReuseKey {
    #[doc(hidden)]
    pub fn same_prepared_input_as_for_test(
        &self,
        other: &Self,
        admit: &mut impl FnMut(
            u64,
        ) -> Result<
            (),
            worth_relational::facade::transactions::RelationalExecutionDenialCause,
        >,
    ) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        Self::same_prepared_input_as(self, other, admit)
    }
}
