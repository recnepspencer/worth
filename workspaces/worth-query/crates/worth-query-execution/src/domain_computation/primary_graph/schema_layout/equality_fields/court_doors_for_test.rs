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

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn equality_field_for_test(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&WorthQueryPrimaryFieldLayout> {
        Self::equality_field(self, entity, aspect, field)
    }
}
