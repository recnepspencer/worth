//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn native_output_aspects_for_test<'a>(
        &'a self,
        entity: &'a str,
    ) -> impl Iterator<Item = &'a AspectKey> + Clone {
        Self::native_output_aspects(self, entity)
    }
}
