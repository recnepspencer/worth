//! A stable ordinary action binds the entire ordered input through native idempotency.
use super::super::schema::*;
use super::{data::*, schema::*};
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
mod handler;
pub use handler::Handler;
#[derive(Debug)]
pub struct Registered;
#[derive(Debug)]
pub enum Denial {
    AttemptKeyMismatch,
    UnsupportedVersion,
    EmptyProposal,
    ModelMismatch,
    ProposalChanged,
    AlreadyClaimed,
    AmbiguousClaim,
}
pub struct Decision {
    model: WorthQueryInvariantMutationTarget<Schema, Model>,
    principal: super::super::schema::data::Coordinates,
}
worth_query_structured_value_binding!(pub InputBinding for AttemptInput {identity:"proposal-application.claim.input.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Registered {identity:"proposal-application.claim.result.v1"});
worth_query_structured_value_binding!(pub DenialBinding for Denial {identity:"proposal-application.claim.denial.v1"});
worth_query_operation!(pub RegisterAttempt for Schema,input InputBinding);
worth_query_operation_reads!(RegisterAttempt=>[ModelName,ModelProposal,ProposalName,ProposalData,ModelClaim,ClaimData]);
worth_query_operation_creates!(RegisterAttempt=>[AcceptanceClaim]);
worth_query_operation_writes!(RegisterAttempt=>[ClaimName,ClaimData]);
worth_query_operation_links!(RegisterAttempt=>[ModelClaim]);
fn model(input: &AttemptInput) -> String {
    input.proposal.model.label.clone()
}
worth_query_mutation_binding!(pub Binding for AttemptInput,schema Schema,
    identity "proposal-application.claim.binding.v1",input InputBinding,operation RegisterAttempt,
    result ResultBinding,idempotency u64,identity "proposal-application.claim.key.v1",
    decision Decision,denial DenialBinding,handler identity "proposal-application.claim.handler.v1",program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),outputs NoApplicationMutationOutputs,
    principal PrincipalBinding,mapping ExternalMapping,principal_entity Principal,
    principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Model,ModelFacts,ModelName,String,ReadOnly,NoApplicationUnit,field ModelName::reference(),value model,
);
pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = RegisterAttempt::reference();
    builder
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 16)
        .operation_read_field(op, ModelName::reference())
        .operation_read_field(op, ProposalName::reference())
        .operation_read_field(op, ProposalData::reference())
        .operation_read_field(op, ClaimData::reference())
        .operation_read_relation(op, ModelProposal::reference())
        .operation_read_relation(op, ModelClaim::reference())
        .operation_create(op, AcceptanceClaim::reference())
        .operation_write(op, ClaimName::reference())
        .operation_write(op, ClaimData::reference())
        .operation_link(op, ModelClaim::reference())
        .application_mutation_binding::<Binding>()
}
