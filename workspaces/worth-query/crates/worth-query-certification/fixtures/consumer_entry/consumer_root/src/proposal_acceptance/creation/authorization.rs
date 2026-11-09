//! New-model creation issues grants using its genuinely observed principal targets.
use super::super::{
    access::{self, grants::*, ReadKind},
    schema::*,
};
use super::{Binding, Create, CreateModel};
use worth_query_decl::facade::{application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

worth_query_operation_creates!(Create=>[Grant]);

worth_query_operation_links!(Create=>[GrantResource,GrantGrantee,GrantGrantor]);

pub(super) fn issue(
    input: &CreateModel,
    registry: &WorthQueryApplicationEffectEntity<Schema, Registry>,
    model: &WorthQueryApplicationEffectEntity<Schema, Model>,
    principals: &[WorthQueryInvariantMutationTarget<Schema, Principal>; 2],
    writer: &mut CandidateWriter<'_, Schema, Binding>,
) -> Result<(), HandlerExecutionDenial> {
    // The host's actual Author principal is this application's grant administrator.
    let grantor = writer
        .projected_entity(&principals[0])
        .map_err(HandlerExecutionDenial::new)?;
    for (principal_number, target) in [1u64, 2].into_iter().zip(principals) {
        let grantee = writer
            .projected_entity(target)
            .map_err(HandlerExecutionDenial::new)?;
        for kind in [ReadKind::Model, ReadKind::Proposal, ReadKind::Result] {
            let key = access::grant_key(&input.name, principal_number, kind);
            let grant = writer
                .create_entity_in_context(
                    registry,
                    Grant::reference(),
                    WorthQueryApplicationEntityKey::new(key.clone())
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&grant, GrantIdentity::at("GrantIdentity"), key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&grant, GrantAction::at("GrantAction"), 1)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&grant, GrantReadKind::at("GrantReadKind"), kind.code())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&grant, GrantPurpose::at("GrantPurpose"), 1)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&grant, GrantStatus::at("GrantStatus"), 1)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&grant, GrantReadStatus::at("GrantReadStatus"), 1)
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
                .initialize_field(&grant, GrantDisclosure::at("GrantDisclosure"), 1)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    GrantResource::reference(),
                    format!("{key}:resource"),
                    &grant,
                    model,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    GrantGrantee::reference(),
                    format!("{key}:grantee"),
                    &grantee,
                    &grant,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    GrantGrantor::reference(),
                    format!("{key}:grantor"),
                    &grantor,
                    &grant,
                )
                .map_err(HandlerExecutionDenial::new)?;
        }
    }
    Ok(())
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = Create::reference();
    schema
        .operation_create(op, Grant::reference())
        .operation_write(op, GrantIdentity::at("GrantIdentity"))
        .operation_write(op, GrantAction::at("GrantAction"))
        .operation_write(op, GrantReadKind::at("GrantReadKind"))
        .operation_write(op, GrantPurpose::at("GrantPurpose"))
        .operation_write(op, GrantStatus::at("GrantStatus"))
        .operation_write(op, GrantReadStatus::at("GrantReadStatus"))
        .operation_write(op, GrantScope::at("GrantScope"))
        .operation_write(op, GrantNotBefore::at("GrantNotBefore"))
        .operation_write(op, GrantNotAfter::at("GrantNotAfter"))
        .operation_write(op, GrantDepth::at("GrantDepth"))
        .operation_write(op, GrantDisclosure::at("GrantDisclosure"))
        .operation_link(op, GrantResource::reference())
        .operation_link(op, GrantGrantee::reference())
        .operation_link(op, GrantGrantor::reference())
}
