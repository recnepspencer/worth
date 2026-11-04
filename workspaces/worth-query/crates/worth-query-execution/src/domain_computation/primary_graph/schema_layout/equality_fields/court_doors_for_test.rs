//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn equality_fields_mut_for_test(
        &mut self,
    ) -> impl Iterator<Item = (&(String, String, String), &mut WorthQueryPrimaryFieldLayout)> {
        Self::equality_fields_mut(self)
    }
}
