//! The stored authored edit is also the ordinary installed mutation input.
use super::{
    mutation_access,
    schema::{data::RetentionEdit, *},
};
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
mod handler;
pub mod rule;
pub use handler::Handler;

#[derive(Debug)]
pub struct Changed {
    pub days: u64,
}
#[derive(Debug)]
pub enum Denial {
    ForeignRuntime,
    ModelOccurrence,
    TargetMembership,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct StepKey {
    pub attempt: u64,
    pub ordinal: u64,
}
worth_query_structured_value_binding!(pub InputBinding for RetentionEdit {identity:"proposal-application.retention.input.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Changed {identity:"proposal-application.retention.result.v1"});
worth_query_structured_value_binding!(pub DenialBinding for Denial {identity:"proposal-application.retention.denial.v1"});
worth_query_operation!(pub SetRetention for Schema,input InputBinding);
worth_query_operation_reads!(SetRetention=>[ModelName,ModelTarget,TargetName,Retention]);
worth_query_operation_writes!(SetRetention=>[Retention]);
fn model(input: &RetentionEdit) -> String {
    input.model.label.clone()
}
worth_query_mutation_binding!(pub Binding for RetentionEdit,schema Schema,
    identity "proposal-application.retention.binding.v1",input InputBinding,operation SetRetention,
    result ResultBinding,idempotency StepKey,identity "proposal-application.retention.key.v2",
    decision WorthQueryInvariantMutationTarget<Schema,Target>,denial DenialBinding,
    handler identity "proposal-application.retention.handler.v1",
    admission ApplicationCapabilityAdmission<mutation_access::EditRetention>,program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),outputs NoApplicationMutationOutputs,
    principal PrincipalBinding,mapping ExternalMapping,principal_entity Principal,
    principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Model,ModelFacts,ModelName,String,ReadOnly,NoApplicationUnit,
    field ModelName::reference(),value model,
);
impl ApplicationCapabilityMutationBinding<Schema> for Binding {
    type Capability = mutation_access::EditRetention;
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = SetRetention::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 16)
        .operation_read_field(op, ModelName::reference())
        .operation_read_relation(op, ModelTarget::reference())
        .operation_read_field(op, TargetName::reference())
        .operation_read_field(op, Retention::reference())
        .operation_write(op, Retention::reference())
        .application_mutation_binding::<Binding>()
        .invariant(rule::definition())
}
