//! New-model creation issues grants using its genuinely observed principal targets.
use super::super::{
    access::{self, grants::*, ReadKind},
    schema::*,
};
use super::{Binding, Create, CreateModel};
use worth_query_decl::facade::{application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

worth_query_operation_creates!(Create=>[Grant]);
worth_query_operation_writes!(Create=>[GrantIdentity,GrantAction,GrantReadKind,GrantPurpose,GrantStatus,
    GrantReadStatus,GrantScope,GrantNotBefore,GrantNotAfter,GrantDepth,GrantDisclosure]);
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
                .initialize_field(&grant, GrantIdentity::reference(), key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&grant, GrantAction::reference(), 1)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&grant, GrantReadKind::reference(), kind.code())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&grant, GrantPurpose::reference(), 1)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&grant, GrantStatus::reference(), 1)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&grant, GrantReadStatus::reference(), 1)
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
                .initialize_field(&grant, GrantDisclosure::reference(), 1)
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
        .operation_write(op, GrantIdentity::reference())
        .operation_write(op, GrantAction::reference())
        .operation_write(op, GrantReadKind::reference())
        .operation_write(op, GrantPurpose::reference())
        .operation_write(op, GrantStatus::reference())
        .operation_write(op, GrantReadStatus::reference())
        .operation_write(op, GrantScope::reference())
        .operation_write(op, GrantNotBefore::reference())
        .operation_write(op, GrantNotAfter::reference())
        .operation_write(op, GrantDepth::reference())
        .operation_write(op, GrantDisclosure::reference())
        .operation_link(op, GrantResource::reference())
        .operation_link(op, GrantGrantee::reference())
        .operation_link(op, GrantGrantor::reference())
}
