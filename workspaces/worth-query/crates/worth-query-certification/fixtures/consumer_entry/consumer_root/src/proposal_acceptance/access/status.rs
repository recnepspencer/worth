//! Only the actual grantor can change a grant's current disclosure/READ facts.
use super::{grants::*, *};
use worth_query_decl::facade::application_operation::*;
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, serde::Serialize)]
pub struct SetStatus {
    pub identity: String,
    pub active: bool,
    pub readable: bool,
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {
    Unchanged,
}
worth_query_structured_value_binding!(pub InputBinding for SetStatus {identity:"proposal-application.grant-status.input.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Changed {identity:"proposal-application.grant-status.result.v1"});
worth_query_structured_value_binding!(pub DenialBinding for Denial {identity:"proposal-application.grant-status.denial.v1"});
worth_query_operation!(pub ChangeGrant for Schema,input InputBinding);
worth_query_operation_reads!(ChangeGrant=>[GrantIdentity,GrantStatus,GrantReadStatus]);
worth_query_operation_writes!(ChangeGrant=>[GrantStatus,GrantReadStatus]);
worth_query_operation_requires!(ChangeGrant=>[ManageGrants]);
fn key(input: &SetStatus) -> String {
    input.identity.clone()
}
worth_query_mutation_binding!(pub Binding for SetStatus,schema Schema,
    identity "proposal-application.grant-status.binding.v1",input InputBinding,operation ChangeGrant,
    result ResultBinding,idempotency u64,identity "proposal-application.grant-status.key.v1",
    decision WorthQueryInvariantMutationTarget<Schema,Grant>,denial DenialBinding,
    handler identity "proposal-application.grant-status.handler.v1",program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),outputs NoApplicationMutationOutputs,
    principal PrincipalBinding,mapping ExternalMapping,principal_entity Principal,
    principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Grant,GrantFacts,GrantIdentity,String,ReadOnly,NoApplicationUnit,
    field GrantIdentity::reference(),value key,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &SetStatus,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Grant>, Denial> {
        let mut read = || -> Result<_, HandlerExecutionDenial> {
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
        match read() {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(Denial::Unchanged),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &SetStatus,
        target: WorthQueryInvariantMutationTarget<Schema, Grant>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Changed, Denial> {
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
            Ok(()) => HandlerResult::Completed(Changed),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = ChangeGrant::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_requires_ability(op, ManageGrants::reference())
        .operation_projection_work_budget(op, 16)
        .operation_read_field(op, GrantIdentity::reference())
        .operation_read_field(op, GrantStatus::reference())
        .operation_read_field(op, GrantReadStatus::reference())
        .operation_write(op, GrantStatus::reference())
        .operation_write(op, GrantReadStatus::reference())
        .application_mutation_binding::<Binding>()
}
