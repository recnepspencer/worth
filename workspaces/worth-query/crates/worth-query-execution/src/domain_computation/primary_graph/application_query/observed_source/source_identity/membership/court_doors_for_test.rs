//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryObservedSourceSelection {
    #[doc(hidden)]
    pub fn new_for_test(
        runtime_authority: u64,
        schema_binding: ApplicationSchemaBindingIdentity,
        meaning: Arc<WorthQueryObservedSourceMeaning>,
        managed: Option<Arc<ManagedSourceRead>>,
    ) -> Self {
        Self {
            runtime_authority,
            schema_binding,
            meaning,
            managed,
        }
    }
}

impl WorthQueryObservedSourceSelection {
    #[doc(hidden)]
    pub fn same_selected_membership_as_for_test(
        &self,
        other: &Self,
        admit: &mut impl FnMut(u64) -> Result<(), Cause>,
    ) -> Result<bool, Cause> {
        Self::same_selected_membership_as(self, other, admit)
    }
}
