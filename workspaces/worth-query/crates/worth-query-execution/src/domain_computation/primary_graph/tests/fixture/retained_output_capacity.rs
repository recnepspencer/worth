//! Real committed output read with all remaining invalidation capacity held.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryProducerApplicability, WorthQueryProducerOutputFamily,
};

// This operation isolates retained capacity. Work remains absent at both the
// installed operation and native marking owner, independently of retained bytes.
worth_query_operation!(pub(in crate::domain_computation::primary_graph) ReadRetainedAccount for IdentityExecutionSchema, input TouchAccountInputBinding);
worth_query_operation_reads!(ReadRetainedAccount => [Account]);
worth_query_operation_requires!(ReadRetainedAccount => [ViewAccount]);

pub(super) fn declare(
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

pub(in crate::domain_computation::primary_graph) struct RetainedFamily;
impl WorthQueryProducerOutputFamily<IdentityExecutionSchema> for RetainedFamily {
    type Source = TestAccountSourceBinding;
    type Entity = Account;
    const IDENTITY: &'static str = "test.retained-account-capacity";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[];
    fn profile_kind(_: &AccountSummaryResult) -> &'static str {
        "retained"
    }
}
