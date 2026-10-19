//! Row edit, parent revision and audit share one installed native value and commit.
use super::{path_value_binding::BumpBinding, revision_query, schema::*};
use serde::Serialize;
use worth_query_decl::facade::{
    application_query::ApplicationQueryRootPath, application_schema::*, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_reads, worth_query_operation_writes,
};
#[derive(Clone, Serialize)]
pub struct BumpRow {
    pub row: String,
    pub value: u64,
}
pub struct BumpInput;
impl ApplicationStructuredValueBinding for BumpInput {
    type Value = BumpRow;
    const IDENTITY_NAME: &'static str = "worth.query.path-value.input.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &BumpRow) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
worth_query_operation!(pub Bump for Schema, input BumpInput);
worth_query_operation_reads!(Bump => [Row, Parent, ParentRevision, ParentRows, RowIdentity]);
worth_query_operation_writes!(Bump => [ParentRevision, RowValue, AuditIdentity, AuditValue]);
worth_query_operation_creates!(Bump => [Audit]);
pub fn body() -> ApplicationOperationBody<Schema, Bump, BumpRow> {
    let op = Bump::reference();
    let operands = op
        .requirement_operands()
        .query_context::<BumpBinding, _, _, _, _>("context", revision_query::definition())
        .unwrap();
    let expression = worth_foundational::expression_api::expressions()
        .reading_within(ApplicationOperationBodyExpression::PROFILE)
        .parse("context.revision + uint64(1)")
        .unwrap();
    let next = ApplicationOperationBodyValue::<Schema, Bump, U64ApplicationValueBinding>::admit(
        "next-revision",
        operands,
        expression,
    )
    .unwrap();
    op.body()
        .assign_value_at_path(
            ApplicationQueryRootPath::from(Row::reference()).reverse(ParentRows::reference()),
            ParentRevision::reference(),
            &next,
        )
        .unwrap()
        .assign_input(RowValue::reference(), "value")
        .unwrap()
        .create_in_scope(Audit::reference(), "revision-audit")
        .unwrap()
        .initialize_input(AuditIdentity::reference(), "row")
        .unwrap()
        .initialize_value(AuditValue::reference(), &next)
        .unwrap()
        .finish()
        .unwrap()
}
