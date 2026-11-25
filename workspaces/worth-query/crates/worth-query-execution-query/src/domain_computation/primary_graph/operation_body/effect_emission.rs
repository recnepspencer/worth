//! Bind a body's ordinary effect recipe once at native installation; only the
//! typed declaration that registered it can supply its payload derivation.
use super::super::{
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use std::any::TypeId;
use std::sync::Arc;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyEffectRecipe as Recipe, ApplicationOperationBodyProgram,
    ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

pub(super) fn install<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    body: &ApplicationOperationBodyProgram,
) -> Result<Option<Arc<Recipe>>, Denial> {
    let Some(emission) = body.effect_emission() else {
        return Ok(None);
    };
    let denied = || Denial::new(Kind::OperationBodyUnavailable, Binding::IDENTITY);
    // A signed root reserves its provenance slot after every ordinary effect,
    // so emitting bodies stay ordinary, like body external emission.
    if Binding::WORKFLOW_CONTROL
        || body.succession().is_some()
        || body
            .creation()
            .is_some_and(|creation| creation.signing().is_some() || creation.enrollment().is_some())
    {
        return Err(denied());
    }
    let recipe = schema
        .body_effect_recipe(Binding::Operation::IDENTIFIER)
        .ok_or_else(denied)?;
    if recipe.meaning() != emission
        || recipe.input_type() != TypeId::of::<Binding::Input>()
        || recipe.operation_type() != TypeId::of::<Binding::Operation>()
        || emission.input_binding != Binding::InputBinding::IDENTITY
    {
        return Err(denied());
    }
    Ok(Some(Arc::new(recipe.clone())))
}
