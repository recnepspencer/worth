//! Actual grant-status transitions drive independent action and facet decisions.
use super::super::schema::*;
use super::access::grants::*;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_mutation_binding,
    worth_query_operation, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, serde::Serialize)]
pub struct SetStatus {
    pub identity: String,
    pub active: bool,
    pub readable: bool,
}
#[derive(Debug)]
pub struct StatusChanged;
#[derive(Debug)]
pub enum Denial {
    Unchanged,
}
worth_query_structured_value_binding!(pub SetStatusInputBinding for SetStatus { identity: "managed-catalog.box-grant.status-input.v1" });
worth_query_structured_value_binding!(pub StatusChangedBinding for StatusChanged { identity: "managed-catalog.box-grant.status-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "managed-catalog.box-grant.status-denial.v1" });
worth_query_operation!(pub SetGrantStatus for Catalog, input SetStatusInputBinding);
worth_query_operation_reads!(SetGrantStatus => [GrantIdentity, GrantStatus, GrantReadStatus]);
worth_query_operation_writes!(SetGrantStatus => [GrantStatus, GrantReadStatus]);
fn identity(input: &SetStatus) -> String {
    input.identity.clone()
}
worth_query_mutation_binding!(
    pub Binding for SetStatus, schema Catalog, identity "managed-catalog.box-grant.status-binding.v1",
    input SetStatusInputBinding, operation SetGrantStatus, result StatusChangedBinding,
    idempotency u64, identity "managed-catalog.box-grant.status-key.v1",
    decision WorthQueryInvariantMutationTarget<Catalog, BoxGrant>, denial DenialBinding,
    handler identity "managed-catalog.box-grant.status-handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope BoxGrant, BoxGrantFacts, GrantIdentity, String, ReadOnly, NoApplicationUnit,
    field GrantIdentity::reference(), value identity,
);
pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &SetStatus,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, BoxGrant>, Denial> {
        let mut observe = || -> Result<_, HandlerExecutionDenial> {
            let grant =
                reader.resolve_entity(GrantIdentity::reference(), input.identity.clone())?;
            let active = reader.field(&grant, GrantStatus::reference())?;
            let readable = reader.field(&grant, GrantReadStatus::reference())?;
            if active == Some(u64::from(input.active))
                && readable == Some(u64::from(input.readable))
            {
                return Ok(None);
            }
            reader.mutation_target(&grant).map(Some)
        };
        match observe() {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(Denial::Unchanged),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &SetStatus,
        target: WorthQueryInvariantMutationTarget<Catalog, BoxGrant>,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<StatusChanged, Denial> {
        let mut apply = || -> Result<(), HandlerExecutionDenial> {
            let grant = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&grant, GrantStatus::reference(), u64::from(input.active))
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(
                    &grant,
                    GrantReadStatus::reference(),
                    u64::from(input.readable),
                )
                .map_err(HandlerExecutionDenial::new)
        };
        match apply() {
            Ok(()) => HandlerResult::Completed(StatusChanged),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let op = SetGrantStatus::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 16)
        .operation_read_field(op, GrantIdentity::reference())
        .operation_read_field(op, GrantStatus::reference())
        .operation_read_field(op, GrantReadStatus::reference())
        .operation_write(op, GrantStatus::reference())
        .operation_write(op, GrantReadStatus::reference())
        .application_mutation_binding::<Binding>()
}
