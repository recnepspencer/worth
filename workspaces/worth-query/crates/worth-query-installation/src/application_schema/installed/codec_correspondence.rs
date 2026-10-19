//! Installed codec correspondence is distinct from portable descriptive meaning.
use super::{
    WorthQueryInstalledApplicationSchema, WorthQueryInstalledApplicationValueBindingCatalog,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyExternalEffectRecipe, ApplicationSchema,
};
impl<Schema: ApplicationSchema> WorthQueryInstalledApplicationSchema<Schema> {
    /// Returns the scalar codecs validated for this exact installed generation.
    pub fn value_bindings(&self) -> &WorthQueryInstalledApplicationValueBindingCatalog {
        &self.value_binding_catalog
    }
    /// Resolves executable codecs from the native typed declaration. Portable
    /// names and decoded bodies alone cannot provide this correspondence.
    pub fn body_external_effect_recipe(
        &self,
        operation: &str,
    ) -> Option<&ApplicationOperationBodyExternalEffectRecipe> {
        self.member_provenance.body_external_effect(operation)
    }
}
