//! Compiler-local body effect recipes: correspondence for an installed body's
//! external and ordinary effect payloads, never an emission permit.
use super::ApplicationSchemaMemberProvenance;

impl ApplicationSchemaMemberProvenance {
    pub(in crate::application_schema) fn register_body_external_effect(
        &mut self,
        recipe: crate::application_schema::ApplicationOperationBodyExternalEffectRecipe,
    ) {
        self.body_external_effects.push(recipe);
    }
    /// A codec is compiler-local correspondence, never an emission permit.
    pub fn body_external_effect(
        &self,
        operation: &str,
    ) -> Option<&crate::application_schema::ApplicationOperationBodyExternalEffectRecipe> {
        let mut recipes = self
            .body_external_effects
            .iter()
            .filter(|recipe| recipe.operation() == operation);
        let recipe = recipes.next()?;
        if recipes.next().is_some() {
            return None;
        }
        (self.operations.iter().any(|member| {
            member.name == operation
                && member.marker_type == recipe.operation_type()
                && member.value_type == recipe.input_type()
                && member.value_identity == recipe.meaning().input_binding
        }) && self.effects.iter().any(|member| {
            member.name == recipe.meaning().effect
                && member.marker_type == recipe.effect_type()
                && member.value_type == recipe.payload_type()
                && member.value_identity == recipe.meaning().payload_binding
        }))
        .then_some(recipe)
    }

    pub(in crate::application_schema) fn register_body_effect(
        &mut self,
        recipe: crate::application_schema::ApplicationOperationBodyEffectRecipe,
    ) {
        self.body_effects.push(recipe);
    }

    /// The ordinary effect recipe of `operation`'s body, when its operation and
    /// effect markers are the declared members it names.
    pub fn body_effect(
        &self,
        operation: &str,
    ) -> Option<&crate::application_schema::ApplicationOperationBodyEffectRecipe> {
        let mut recipes = self
            .body_effects
            .iter()
            .filter(|recipe| recipe.operation() == operation);
        let recipe = recipes.next()?;
        if recipes.next().is_some() {
            return None;
        }
        (self.operations.iter().any(|member| {
            member.name == operation
                && member.marker_type == recipe.operation_type()
                && member.value_type == recipe.input_type()
                && member.value_identity == recipe.meaning().input_binding
        }) && self.effects.iter().any(|member| {
            member.name == recipe.meaning().effect
                && member.marker_type == recipe.effect_type()
                && member.value_type == recipe.payload_type()
                && member.value_identity == recipe.meaning().payload_binding
        }))
        .then_some(recipe)
    }
}
