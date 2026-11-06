//! New-model creation issues independent mutation grants to actual observed principals.
use super::super::{
    mutation_access::{self, grants::*},
    schema::*,
};
use super::{Binding, Create, CreateModel};
use worth_query_decl::facade::{application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
worth_query_operation_creates!(Create=>[MutationGrant]);

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
            .initialize_field(&grant, GrantIdentity::at("GrantIdentity"), key.clone())
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantAction::at("GrantAction"), 2)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantPurpose::at("GrantPurpose"), 1)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantStatus::at("GrantStatus"), 1)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantScope::at("GrantScope"), input.name.clone())
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantNotBefore::at("GrantNotBefore"), 0)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantNotAfter::at("GrantNotAfter"), u64::MAX)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&grant, GrantDepth::at("GrantDepth"), 0)
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
        .operation_write(op, GrantIdentity::at("GrantIdentity"))
        .operation_write(op, GrantAction::at("GrantAction"))
        .operation_write(op, GrantPurpose::at("GrantPurpose"))
        .operation_write(op, GrantStatus::at("GrantStatus"))
        .operation_write(op, GrantScope::at("GrantScope"))
        .operation_write(op, GrantNotBefore::at("GrantNotBefore"))
        .operation_write(op, GrantNotAfter::at("GrantNotAfter"))
        .operation_write(op, GrantDepth::at("GrantDepth"))
        .operation_link(op, MutationGrantResource::reference())
        .operation_link(op, MutationGrantGrantee::reference())
        .operation_link(op, MutationGrantGrantor::reference())
}
