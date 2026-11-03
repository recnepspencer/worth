//! Real committed output read with all remaining invalidation capacity held.
use crate::domain_computation::primary_graph::tests::fixture::*;
use worth_query_execution::facade::application_contribution::WorthQueryProducerApplicability;
use worth_query_execution::facade::application_contribution::WorthQueryProducerOutputFamily;

// This operation isolates retained capacity. Work remains absent at both the
// installed operation and native marking owner, independently of retained bytes.
worth_query_operation!(pub ReadRetainedAccount for IdentityExecutionSchema, input TouchAccountInputBinding);
worth_query_operation_reads!(ReadRetainedAccount => [Account]);
worth_query_operation_requires!(ReadRetainedAccount => [ViewAccount]);

pub fn declare(
    schema: worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder<
    IdentityExecutionSchema,
> {
    let operation = ReadRetainedAccount::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_entity(operation, Account::reference())
        .operation_requires_ability(operation, ViewAccount::reference())
}

pub struct RetainedFamily;
impl WorthQueryProducerOutputFamily<IdentityExecutionSchema> for RetainedFamily {
    type Source = TestAccountSourceBinding;
    type Entity = Account;
    const IDENTITY: &'static str = "test.retained-account-capacity";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[];
    fn profile_kind(_: &AccountSummaryResult) -> &'static str {
        "retained"
    }
}
