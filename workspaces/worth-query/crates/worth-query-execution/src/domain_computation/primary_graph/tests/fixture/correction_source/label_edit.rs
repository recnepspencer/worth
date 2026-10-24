//! One installed label-edit handler shared by an original action and its inverse.
use super::super::*;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};
use worth_query_declaration::facade::{application_operation::*, application_schema::*};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(in crate::domain_computation::primary_graph) struct LabelEdit {
    pub status: String,
    pub label: String,
}
worth_query_declaration::worth_query_portable_type!(LabelEdit => "worth.query.test.label-edit.input.v1");
worth_query_declaration::worth_query_structured_value_binding!(
    pub(in crate::domain_computation::primary_graph) LabelInput for LabelEdit {
        identity: "worth.query.test.label-edit.input.v1"
    }
);
worth_query_declaration::worth_query_operation!(
    pub(in crate::domain_computation::primary_graph) EditLabel for IdentityExecutionSchema,
    input LabelInput
);
worth_query_declaration::worth_query_operation_requires!(EditLabel => [ViewAccount]);
worth_query_declaration::worth_query_operation_reads!(EditLabel => [AccountStatus, AccountLabel]);
worth_query_declaration::worth_query_operation_writes!(EditLabel => [AccountLabel]);
worth_query_declaration::worth_query_mutation_binding!(
    pub(in crate::domain_computation::primary_graph) LabelBinding for LabelEdit,
    schema IdentityExecutionSchema,
    identity "worth.query.test.label-edit.binding.v1",
    input LabelInput,
    operation EditLabel,
    result LabelInput,
    idempotency String, identity "worth.query.test.label-edit.key.v1",
    decision LabelEdit,
    denial LabelInput,
    handler identity "worth.query.test.label-edit.handler.v1",
    program required,
    outputs NoApplicationMutationOutputs,
    principal IdentityBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Account, AccountPolicy, AccountStatus, String, ReadWrite, NoApplicationUnit,
    field AccountStatus::reference(), value label_scope,
);
fn label_scope(input: &LabelEdit) -> String {
    input.status.clone()
}
impl ApplicationDeclaredOperation<IdentityExecutionSchema> for EditLabel {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
        schema
            .operation_projection_work_budget(Self::reference(), 16)
            .operation_requires_ability(Self::reference(), ViewAccount::reference())
            .operation_read_field(Self::reference(), AccountStatus::reference())
            .operation_read_field(Self::reference(), AccountLabel::reference())
            .operation_write(Self::reference(), AccountLabel::reference())
    }
}

pub(super) struct LabelHandler;
impl OperationHandler<IdentityExecutionSchema, LabelBinding> for LabelHandler {
    fn decide(
        &self,
        input: &LabelEdit,
        reader: &mut DecisionReader<'_, '_, '_, IdentityExecutionSchema, LabelBinding>,
    ) -> HandlerResult<LabelEdit, LabelEdit> {
        let account = match reader.resolve_entity(AccountStatus::reference(), input.status.clone())
        {
            Ok(account) => account,
            Err(denial) => return HandlerResult::ExecutionDenied(denial),
        };
        match reader.field(&account, AccountLabel::reference()) {
            Ok(Some(_)) => HandlerResult::Completed(input.clone()),
            Ok(None) => HandlerResult::DomainDenied(input.clone()),
            Err(denial) => HandlerResult::ExecutionDenied(denial),
        }
    }
    fn build_candidate(
        &self,
        _: &LabelEdit,
        decision: LabelEdit,
        writer: &mut CandidateWriter<'_, IdentityExecutionSchema, LabelBinding>,
    ) -> HandlerResult<LabelEdit, LabelEdit> {
        let written = writer
            .resolve_entity(AccountStatus::reference(), decision.status.clone())
            .and_then(|account| {
                writer.write_field(&account, AccountLabel::reference(), decision.label.clone())
            });
        match written {
            Ok(()) => HandlerResult::Completed(decision),
            Err(denial) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(denial)),
        }
    }
}
