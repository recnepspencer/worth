//! Exact typed content provenance belongs to the ordinary schema provenance owner.
use super::{ApplicationFieldBindingLocus, ApplicationSchemaMemberProvenance};
impl ApplicationSchemaMemberProvenance {
    pub(in crate::application_schema) fn register_content_binding(
        &mut self,
        recipe: crate::application_schema::ApplicationContentBindingRecipe,
    ) {
        self.content_bindings.push(recipe);
    }
    pub(in crate::application_schema) fn register_content_field(
        &mut self,
        locus: ApplicationFieldBindingLocus,
        recipe: crate::application_schema::ApplicationContentBindingRecipe,
    ) {
        self.content_fields.push((locus, recipe));
    }
    pub fn content_field_bindings(
        &self,
    ) -> &[(
        ApplicationFieldBindingLocus,
        crate::application_schema::ApplicationContentBindingRecipe,
    )] {
        &self.content_fields
    }

    pub fn content_bindings(
        &self,
    ) -> &[crate::application_schema::ApplicationContentBindingRecipe] {
        &self.content_bindings
    }
}
