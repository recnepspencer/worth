//! The genuine mutation-grant issuer alone can revoke or restore it.
use super::{grants::*, *};
use worth_query_decl::facade::application_operation::*;
use worth_query_host::facade::primary_graph::*;
#[derive(Clone, Debug, serde::Serialize)]
pub struct SetStatus {
    pub identity: String,
    pub active: bool,
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {
    Unchanged,
}
worth_query_structured_value_binding!(pub InputBinding for SetStatus{identity:"proposal-application.mutation-grant.status-input.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Changed{identity:"proposal-application.mutation-grant.status-result.v1"});
worth_query_structured_value_binding!(pub DenialBinding for Denial{identity:"proposal-application.mutation-grant.status-denial.v1"});
worth_query_operation!(pub ChangeStatus for Schema,input InputBinding);
worth_query_operation_reads!(ChangeStatus=>[GrantIdentity,GrantStatus]);
worth_query_operation_writes!(ChangeStatus=>[GrantStatus]);
worth_query_operation_requires!(ChangeStatus=>[ManageMutationGrants]);
fn identity(input: &SetStatus) -> String {
    input.identity.clone()
}
worth_query_mutation_binding!(pub Binding for SetStatus,schema Schema,
    identity "proposal-application.mutation-grant.status-binding.v1",input InputBinding,operation ChangeStatus,
    result ResultBinding,idempotency u64,identity "proposal-application.mutation-grant.status-key.v1",
    decision WorthQueryInvariantMutationTarget<Schema,MutationGrant>,denial DenialBinding,
    handler identity "proposal-application.mutation-grant.status-handler.v1",program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),outputs NoApplicationMutationOutputs,
    principal PrincipalBinding,mapping ExternalMapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope MutationGrant,MutationGrantFacts,GrantIdentity,String,ReadOnly,NoApplicationUnit,
    field GrantIdentity::reference(),value identity,
);
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &SetStatus,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, MutationGrant>, Denial> {
        let mut read = || -> Result<_, HandlerExecutionDenial> {
            let grant =
                reader.resolve_entity(GrantIdentity::reference(), input.identity.clone())?;
            if reader.field(&grant, GrantStatus::reference())? == Some(u64::from(input.active)) {
                return Ok(None);
            }
            reader.mutation_target(&grant).map(Some)
        };
        match read() {
            Ok(Some(grant)) => HandlerResult::Completed(grant),
            Ok(None) => HandlerResult::DomainDenied(Denial::Unchanged),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &SetStatus,
        target: WorthQueryInvariantMutationTarget<Schema, MutationGrant>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        let effect = writer
            .projected_entity(&target)
            .map_err(HandlerExecutionDenial::new)
            .and_then(|grant| {
                writer
                    .write_field(&grant, GrantStatus::reference(), u64::from(input.active))
                    .map_err(HandlerExecutionDenial::new)
            });
        match effect {
            Ok(()) => HandlerResult::Completed(Changed),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = ChangeStatus::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 16)
        .operation_requires_ability(op, ManageMutationGrants::reference())
        .operation_read_field(op, GrantIdentity::reference())
        .operation_read_field(op, GrantStatus::reference())
        .operation_write(op, GrantStatus::reference())
        .application_mutation_binding::<Binding>()
}
