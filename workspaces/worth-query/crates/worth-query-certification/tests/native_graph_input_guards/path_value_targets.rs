//! Target-only writes protect their destination even when operands came from elsewhere.
use super::{
    actions::{DenialBinding, ResultBinding},
    schema::*,
};
use serde::Serialize;
use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs,
    application_query::ApplicationQueryRootPath, application_schema::*,
    worth_query_mutation_binding, worth_query_operation, worth_query_operation_reads,
    worth_query_operation_writes,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision;
#[derive(Clone, Serialize)]
pub struct CopyRow {
    pub row: String,
}
#[derive(Clone, Serialize)]
pub struct AliasRow {
    pub row: String,
}
pub struct CopyInput;
pub struct AliasInput;
impl ApplicationStructuredValueBinding for CopyInput {
    type Value = CopyRow;
    const IDENTITY_NAME: &'static str = "worth.query.path-value.copy-input.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &CopyRow) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
impl ApplicationStructuredValueBinding for AliasInput {
    type Value = AliasRow;
    const IDENTITY_NAME: &'static str = "worth.query.path-value.alias-input.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &AliasRow) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
worth_query_operation!(pub Copy for Schema, input CopyInput);
worth_query_operation!(pub Alias for Schema, input AliasInput);
worth_query_operation_reads!(Copy=>[Row,Parent,ParentRevision,ParentRows,RowValue,RowIdentity]);
worth_query_operation_reads!(Alias=>[Row,Parent,ParentRevision,ParentRows,RowValue,RowIdentity]);
worth_query_operation_writes!(Copy=>[ParentRevision]);
worth_query_operation_writes!(Alias=>[ParentRevision]);
fn copy_row(input: &CopyRow) -> String {
    input.row.clone()
}
fn alias_row(input: &AliasRow) -> String {
    input.row.clone()
}
worth_query_mutation_binding!(pub CopyBinding for CopyRow, schema Schema,
    identity "worth.query.path-value.copy-binding.v1", input CopyInput, operation Copy, result ResultBinding,
    idempotency u64, identity "worth.query.path-value.copy-request.v1",
    decision WorthQueryOperationBodyDecision<Schema, Row>, denial DenialBinding,
    handler identity "worth.query.path-value.copy-body.v1", program required, outputs NoApplicationMutationOutputs,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Row, RowFacts, RowIdentity, String, ReadOnly, NoApplicationUnit, field RowIdentity::reference(), value copy_row,
    candidates creates 0, deletes 0, links 0, unlinks 0, writes 1, emits 0, resources retained_representation_bytes 4096);
worth_query_mutation_binding!(pub AliasBinding for AliasRow, schema Schema,
    identity "worth.query.path-value.alias-binding.v1", input AliasInput, operation Alias, result ResultBinding,
    idempotency u64, identity "worth.query.path-value.alias-request.v1",
    decision WorthQueryOperationBodyDecision<Schema, Row>, denial DenialBinding,
    handler identity "worth.query.path-value.alias-body.v1", program required, outputs NoApplicationMutationOutputs,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Row, RowFacts, RowIdentity, String, ReadOnly, NoApplicationUnit, field RowIdentity::reference(), value alias_row,
    candidates creates 0, deletes 0, links 0, unlinks 0, writes 2, emits 0, resources retained_representation_bytes 4096);
fn value<O, I>(
    operation: ApplicationOperationRef<Schema, O, I>,
) -> ApplicationOperationBodyValue<Schema, O, U64ApplicationValueBinding>
where
    RowValue: OperationReads<O>,
{
    ApplicationOperationBodyValue::admit(
        "row-value",
        operation
            .requirement_operands()
            .field("value", RowValue::reference())
            .unwrap(),
        worth_foundational::expression_api::expressions()
            .reading_within(ApplicationOperationBodyExpression::PROFILE)
            .parse("value")
            .unwrap(),
    )
    .unwrap()
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let copy = Copy::reference();
    let copied = value(copy);
    let body = copy
        .body()
        .assign_value_at_path(
            ApplicationQueryRootPath::from(Row::reference()).reverse(ParentRows::reference()),
            ParentRevision::reference(),
            &copied,
        )
        .unwrap()
        .finish()
        .unwrap();
    let alias = Alias::reference();
    let copied = value(alias);
    let alias_body = alias
        .body()
        .assign_value_at_path(
            ApplicationQueryRootPath::from(Row::reference()).reverse(ParentRows::reference()),
            ParentRevision::reference(),
            &copied,
        )
        .unwrap()
        .assign_value_at_path(
            ApplicationQueryRootPath::from(Row::reference())
                .reverse(ParentRows::reference())
                .forward(ParentRows::reference())
                .reverse(ParentRows::reference()),
            ParentRevision::reference(),
            &copied,
        )
        .unwrap()
        .finish()
        .unwrap();
    schema
        .operation(
            copy.definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(body)
                .unwrap(),
        )
        .operation(
            alias
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(alias_body)
                .unwrap(),
        )
        .operation_read_field(copy, RowIdentity::reference())
        .operation_read_field(alias, RowIdentity::reference())
        .operation_projection_work_budget(copy, 256)
        .operation_projection_work_budget(alias, 256)
        .application_mutation_binding::<CopyBinding>()
        .application_mutation_binding::<AliasBinding>()
}
