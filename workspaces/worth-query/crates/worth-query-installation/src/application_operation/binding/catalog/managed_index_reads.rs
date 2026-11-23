//! Resolve declared Parent bindings, own-scope and shared-owner peer reads,
//! against the genuine mutation scope before execution.
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
    for target in worth_query_declaration::facade::application_schema::application_operation_decision_read_targets(
        schema.installed_declaration().members(), mutation.operation_name()) {
        let (parent, peer_member) = match target {
            Read::ManagedIndex { parent } => (parent, None),
            Read::SharedOwnerManagedIndex { parent, member, .. } => (parent, Some(member)),
            _ => continue,
        };
        let query = schema
            .installed_query_binding_inventory()
            .find(|query| parent.matches_descriptor(query))
            .ok_or_else(|| {
                denial(
                    DenialKind::MutationSourceQueryNotInstalled,
                    mutation.identity(),
                )
            })?;
        // A peer scope is named by input and is the same kind as the
        // operation's own scope: the governing relation's member endpoint.
        let matching_mode = match peer_member {
            None => matches!(
                (mutation.scope().mode(), query.scope().mode()),
                (MutationMode::InputField, QueryMode::InputField)
                    | (
                        MutationMode::PrincipalIdentity,
                        QueryMode::PrincipalIdentity
                    )
            ),
            Some(member) => {
                matches!(
                    (mutation.scope().mode(), query.scope().mode()),
                    (MutationMode::InputField, QueryMode::InputField)
                ) && query.scope().field().locus().entity() == member.as_str()
            }
        };
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
