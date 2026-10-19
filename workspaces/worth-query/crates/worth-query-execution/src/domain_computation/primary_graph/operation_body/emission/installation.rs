use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;

pub(in crate::domain_computation::primary_graph::operation_body) fn install<
    Schema: worth_query_installation::facade::ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
>(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
    body: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram,
) -> Result<Option<Arc<Recipe>>, Denial> {
    let Some(emission) = body.external_emission() else {
        return Ok(None);
    };
    let denied = || Denial::new(Kind::OperationBodyUnavailable, Binding::IDENTITY);
    if Binding::WORKFLOW_CONTROL
        || body.succession().is_some()
        || body
            .creation()
            .is_some_and(|creation| creation.signing().is_some() || creation.enrollment().is_some())
    {
        return Err(denied());
    }
    let recipe = schema
        .body_external_effect_recipe(Binding::Operation::IDENTIFIER)
        .ok_or_else(denied)?;
    use worth_query_declaration::facade::application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding,
    };
    if recipe.meaning() != emission
        || recipe.input_type() != std::any::TypeId::of::<Binding::Input>()
        || recipe.operation_type() != std::any::TypeId::of::<Binding::Operation>()
        || emission.input_binding != Binding::InputBinding::IDENTITY
    {
        return Err(denied());
    }
    let payload_type = match &emission.source {
        worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::OperationInput => std::any::TypeId::of::<Binding::Input>(),
        worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::QueryContext { mutation_binding, .. } if mutation_binding == Binding::IDENTITY => std::any::TypeId::of::<worth_query_declaration::facade::application_query::ApplicationQueryExpressionSnapshot>(),
        worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::Evaluation { mutation_binding, .. } if mutation_binding == Binding::IDENTITY && !body.values().is_empty() => std::any::TypeId::of::<worth_query_declaration::facade::application_schema::ApplicationOperationBodyEvaluationSnapshot>(),
        _ => return Err(denied()),
    };
    if recipe.payload_type() != payload_type {
        return Err(denied());
    }
    Ok(Some(Arc::new(recipe.clone())))
}
