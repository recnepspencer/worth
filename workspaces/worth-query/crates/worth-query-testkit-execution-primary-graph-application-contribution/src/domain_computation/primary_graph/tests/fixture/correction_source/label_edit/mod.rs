//! One installed label-edit handler shared by an original action and its inverse.
use crate::domain_computation::primary_graph::tests::fixture::*;
use worth_query_declaration::facade::application_operation::*;
use worth_query_declaration::facade::application_schema::*;
use worth_query_execution::facade::domain_computation::CandidateWriter;
use worth_query_execution::facade::domain_computation::DecisionReader;
use worth_query_execution::facade::domain_computation::HandlerExecutionDenial;
use worth_query_execution::facade::domain_computation::HandlerResult;
use worth_query_execution::facade::domain_computation::OperationHandler;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LabelEdit {
    pub status: String,
    pub label: String,
}
worth_query_declaration::worth_query_portable_type!(LabelEdit => "worth.query.test.label-edit.input.v1");
worth_query_declaration::worth_query_structured_value_binding!(
    pub LabelInput for LabelEdit {
        identity: "worth.query.test.label-edit.input.v1"
    }
);
worth_query_declaration::worth_query_operation!(
    pub EditLabel for IdentityExecutionSchema,
    input LabelInput
);
worth_query_declaration::worth_query_operation_requires!(EditLabel => [ViewAccount]);
worth_query_declaration::worth_query_operation_reads!(EditLabel => [AccountStatus, AccountLabel]);
worth_query_declaration::worth_query_operation_writes!(EditLabel => [AccountLabel]);
worth_query_declaration::worth_query_mutation_binding!(
    pub LabelBinding for LabelEdit,
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
pub fn label_scope(input: &LabelEdit) -> String {
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

pub struct LabelHandler;
impl OperationHandler<IdentityExecutionSchema, LabelBinding> for LabelHandler {
    fn declared_durable_capabilities() -> Option<&'static [worth_query_execution::facade::primary_graph::WorthQueryOperationDurableCapability]>{
        use worth_query_execution::facade::primary_graph::WorthQueryOperationDurableCapability as Capability;
        Some(&[
            Capability::RelationalMutation,
            Capability::RetainedCorrectionMaterial,
        ])
    }
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
