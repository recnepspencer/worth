//! A computation that creates output from a source it changes in the same commit.
use crate::domain_computation::primary_graph::tests::fixture::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputContract;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationDeclaredOutputRole;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputRole;
use worth_query_declaration::facade::application_operation::WorthQueryCreateOutput;
use worth_query_declaration::facade::application_operation::WorthQueryExactlyOneOutput;
use worth_query_declaration::worth_query_operation_creates;
worth_query_operation!(pub OwnWriteComputation for IdentityExecutionSchema, input TouchAccountInputBinding);
worth_query_operation_reads!(OwnWriteComputation => [AccountStatus]);
worth_query_operation_writes!(OwnWriteComputation => [AccountStatus, AccountMembershipTag]);
worth_query_operation_creates!(OwnWriteComputation => [Account]);
worth_query_operation_requires!(OwnWriteComputation => [ViewAccount]);

pub fn declare(
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
        .operation_read_field(operation, AccountStatus::at("AccountStatus"))
        .operation_read_field(operation, AccountLabel::at("AccountLabel"))
        .operation_write(operation, AccountStatus::at("AccountStatus"))
        .operation_write(operation, AccountLabel::at("AccountLabel"))
        .operation_write(operation, AccountIdentity::at("AccountIdentity"))
        .operation_write(operation, AccountMembershipTag::at("AccountMembershipTag"))
        .operation_create(operation, Account::reference())
}

pub struct OwnWriteOutputs;
impl ApplicationMutationOutputContract<IdentityExecutionSchema> for OwnWriteOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<GeneratedAccount as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}
pub struct GeneratedAccount;
impl WorthQueryApplicationOutputRole for GeneratedAccount {
    type Schema = IdentityExecutionSchema;
    type Contract = OwnWriteOutputs;
    type Entity = Account;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "computed-account";
}
