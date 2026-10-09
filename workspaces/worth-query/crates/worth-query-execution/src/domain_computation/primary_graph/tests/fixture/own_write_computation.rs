//! A computation that creates output from a source it changes in the same commit.
use super::*;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationOutputContract, ApplicationMutationOutputRoleDescriptor,
    WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationOutputRole,
    WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
};
use worth_query_declaration::worth_query_operation_creates;
worth_query_operation!(pub(in crate::domain_computation::primary_graph) OwnWriteComputation for IdentityExecutionSchema, input TouchAccountInputBinding);
worth_query_operation_reads!(OwnWriteComputation => [AccountStatus, AccountLabel]);
worth_query_operation_writes!(OwnWriteComputation => [AccountStatus, AccountLabel, AccountIdentity, AccountMembershipTag]);
worth_query_operation_creates!(OwnWriteComputation => [Account]);
worth_query_operation_requires!(OwnWriteComputation => [ViewAccount]);

pub(super) fn declare(
    schema: worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder<
    IdentityExecutionSchema,
> {
    let operation = OwnWriteComputation::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 32)
        .operation_requires_ability(operation, ViewAccount::reference())
        .operation_read_field(operation, AccountStatus::reference())
        .operation_read_field(operation, AccountLabel::reference())
        .operation_write(operation, AccountStatus::reference())
        .operation_write(operation, AccountLabel::reference())
        .operation_write(operation, AccountIdentity::reference())
        .operation_write(operation, AccountMembershipTag::reference())
        .operation_create(operation, Account::reference())
}

pub(in crate::domain_computation::primary_graph) struct OwnWriteOutputs;
impl ApplicationMutationOutputContract<IdentityExecutionSchema> for OwnWriteOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<GeneratedAccount as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}
pub(in crate::domain_computation::primary_graph) struct GeneratedAccount;
impl WorthQueryApplicationOutputRole for GeneratedAccount {
    type Schema = IdentityExecutionSchema;
    type Contract = OwnWriteOutputs;
    type Entity = Account;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "computed-account";
}
