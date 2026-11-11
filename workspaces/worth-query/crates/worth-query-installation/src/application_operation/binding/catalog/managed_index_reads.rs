//! Resolve declared Parent bindings against the genuine mutation scope before execution.
use super::*;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationScopeResolutionMode as MutationMode,
    application_query::ApplicationQueryScopeResolutionMode as QueryMode,
    application_schema::ApplicationOperationDecisionReadTarget as Read,
};

pub(super) fn validate<S: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<S>,
    mutation: &ApplicationMutationBindingDescriptor,
) -> Result<(), WorthQueryApplicationOperationInstallationDenial> {
    for member in schema.installed_declaration().members() {
        let ApplicationSchemaMember::OperationDecisionRead {
            operation,
            target: Read::ManagedIndex { parent },
        } = member
        else {
            continue;
        };
        if operation != mutation.operation_name() {
            continue;
        }
        let query = schema
            .installed_query_binding_inventory()
            .find(|query| parent.matches_descriptor(query))
            .ok_or_else(|| {
                denial(
                    DenialKind::MutationSourceQueryNotInstalled,
                    mutation.identity(),
                )
            })?;
        let matching_mode = matches!(
            (mutation.scope().mode(), query.scope().mode()),
            (MutationMode::InputField, QueryMode::InputField)
                | (
                    MutationMode::PrincipalIdentity,
                    QueryMode::PrincipalIdentity
                )
        );
        if !matching_mode
            || mutation.scope().field() != query.scope().field()
            || mutation.principal().name() != query.principal().name()
        {
            return Err(denial(
                DenialKind::MutationBindingScopeMeaningChanged,
                mutation.identity(),
            ));
        }
    }
    Ok(())
}
