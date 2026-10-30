//! New-model creation issues independent mutation grants to actual observed principals.
use super::super::{
    mutation_access::{self, grants::*},
    schema::*,
};
use super::{Binding, Create, CreateModel};
use worth_query_decl::facade::{application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
worth_query_operation_creates!(Create=>[MutationGrant]);
worth_query_operation_writes!(Create=>[GrantIdentity,GrantAction,GrantPurpose,GrantStatus,GrantScope,GrantNotBefore,GrantNotAfter,GrantDepth]);
worth_query_operation_links!(Create=>[MutationGrantResource,MutationGrantGrantee,MutationGrantGrantor]);
pub(super) fn issue(
    input: &CreateModel,
    registry: &WorthQueryApplicationEffectEntity<Schema, Registry>,
    model: &WorthQueryApplicationEffectEntity<Schema, Model>,
    principals: &[WorthQueryInvariantMutationTarget<Schema, Principal>; 2],
    writer: &mut CandidateWriter<'_, Schema, Binding>,
) -> Result<(), HandlerExecutionDenial> {
    let grantor = writer
        .projected_entity(&principals[0])
        .map_err(HandlerExecutionDenial::new)?;
    for (number, target) in [1u64, 2].into_iter().zip(principals) {
        let grantee = writer
            .projected_entity(target)
            .map_err(HandlerExecutionDenial::new)?;
        let key = mutation_access::grant_key(&input.name, number);
        let grant = writer
            .create_entity_in_context(
                registry,
                MutationGrant::reference(),
                WorthQueryApplicationEntityKey::new(key.clone())
                    .map_err(HandlerExecutionDenial::new)?,
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantIdentity::reference(), key.clone())
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantAction::reference(), 2)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantPurpose::reference(), 1)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantStatus::reference(), 1)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantScope::reference(), input.name.clone())
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantNotBefore::reference(), 0)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantNotAfter::reference(), u64::MAX)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantDepth::reference(), 0)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .link(
                MutationGrantResource::reference(),
                format!("{key}:resource"),
                &grant,
                model,
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .link(
                MutationGrantGrantee::reference(),
                format!("{key}:grantee"),
                &grantee,
                &grant,
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .link(
                MutationGrantGrantor::reference(),
                format!("{key}:grantor"),
                &grantor,
                &grant,
            )
            .map_err(HandlerExecutionDenial::new)?;
    }
    Ok(())
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = Create::reference();
    schema
        .operation_create(op, MutationGrant::reference())
        .operation_write(op, GrantIdentity::reference())
        .operation_write(op, GrantAction::reference())
        .operation_write(op, GrantPurpose::reference())
        .operation_write(op, GrantStatus::reference())
        .operation_write(op, GrantScope::reference())
        .operation_write(op, GrantNotBefore::reference())
        .operation_write(op, GrantNotAfter::reference())
        .operation_write(op, GrantDepth::reference())
        .operation_link(op, MutationGrantResource::reference())
        .operation_link(op, MutationGrantGrantee::reference())
        .operation_link(op, MutationGrantGrantor::reference())
}
