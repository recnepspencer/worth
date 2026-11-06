//! Sealed test doors over this module's private members.
use super::*;

impl super::super::WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn relation_field_locator_for_test(
        &self,
        relation: &str,
        aspect: &str,
        field: &str,
    ) -> Option<&AspectFieldLocator> {
        Self::relation_field_locator(self, relation, aspect, field)
    }
}
