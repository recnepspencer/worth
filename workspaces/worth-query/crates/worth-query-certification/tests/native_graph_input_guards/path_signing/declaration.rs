//! Native captures and path assignments compose once; no test executor builds effects.
use super::{
    super::{revision_query, schema::*},
    binding::*,
    schema::*,
};
use crate::peer_audit::{Journal, JournalActor, JournalName, JournalRevision, ParentJournal};
use worth_query_decl::facade::{
    application_query::ApplicationQueryRootPath, application_schema::*, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_structured_value_binding,
};
macro_rules! operation {
    ($input:ident,$input_binding:ident,$operation:ident,$entity:ty,$history:ty,[$($field:ty),+])=>{
        #[derive(Clone,serde::Serialize)]pub struct $input(pub String);
        worth_query_structured_value_binding!(pub $input_binding for $input {identity:concat!("worth.query.signed-path.input.",stringify!($input)),capture_input:true});
        worth_query_operation!(pub $operation for Schema,input $input_binding);
        worth_query_operation_reads!($operation=>[Row,RowIdentity,RowValue,Parent,ParentRows,ParentRevision,Company]);
        worth_query_operation_writes!($operation=>[$($field),+]);
        worth_query_operation_creates!($operation=>[$entity]);
        worth_query_operation_links!($operation=>[$history]);
    };
}
operation!(
    SignGraph,
    GraphInput,
    GraphSign,
    GraphEvidence,
    GraphHistory,
    [
        ParentRevision,
        SignedValue,
        SignedRevision,
        SignedCompany,
        GraphActor,
        GraphProvenance
    ]
);
operation!(
    SignGraphAlias,
    GraphAliasInput,
    GraphAliasSign,
    GraphEvidence,
    GraphHistory,
    [
        ParentRevision,
        SignedValue,
        SignedRevision,
        SignedCompany,
        GraphActor,
        GraphProvenance
    ]
);
operation!(
    SignScopeAlias,
    ScopeAliasInput,
    ScopeAliasSign,
    ScopeEvidence,
    ScopeHistory,
    [RowValue, ScopeValue, ScopeActor, ScopeProvenance]
);
worth_query_operation_creates!(GraphSign=>[Journal]);
worth_query_operation_links!(GraphSign=>[ParentJournal]);
worth_query_operation_writes!(GraphSign=>[JournalName,JournalRevision,JournalActor]);
pub fn parent() -> ApplicationQueryRootPath<Schema, Row, Parent> {
    ApplicationQueryRootPath::from(Row::reference()).reverse(ParentRows::reference())
}
pub fn graph_body() -> ApplicationOperationBody<Schema, GraphSign, SignGraph> {
    let op = GraphSign::reference();
    let next =
        ApplicationOperationBodyValue::<Schema, GraphSign, U64ApplicationValueBinding>::admit(
            "next-revision",
            op.requirement_operands()
                .query_context::<GraphBinding, _, _, _, _>("context", revision_query::definition())
                .unwrap(),
            worth_foundational::expression_api::expressions()
                .reading_within(ApplicationOperationBodyExpression::PROFILE)
                .parse("context.revision + uint64(1)")
                .unwrap(),
        )
        .unwrap();
    op.body()
        .in_scope(Row::reference())
        .unwrap()
        .assign_value_at_path(parent(), ParentRevision::reference(), &next)
        .unwrap()
        .create_in_scope(GraphEvidence::reference(), "verification")
        .unwrap()
        .capture_from_scope(
            RowValue::reference(),
            SignedValue::reference(),
            ApplicationOperationBodyCaptureRole::Subject,
        )
        .unwrap()
        .capture_from_path(
            parent(),
            Company::reference(),
            SignedCompany::reference(),
            ApplicationOperationBodyCaptureRole::Subject,
        )
        .unwrap()
        .capture_from_path(
            parent(),
            ParentRevision::reference(),
            SignedRevision::reference(),
            ApplicationOperationBodyCaptureRole::AuditContext,
        )
        .unwrap()
        .signed_by::<GraphBinding, _, _, _, _, _, _, _>(
            GraphActor::reference(),
            GraphProvenance::reference(),
            "path-verification",
        )
        .unwrap()
        .link_from_scope(GraphHistory::reference(), "history")
        .unwrap()
        .with_peer(Journal::reference(), "signed-audit", |audit| {
            audit
                .initialize(JournalName::reference(), "signed".to_string())?
                .initialize_value(JournalRevision::reference(), &next)?
                .initialize_from_principal_identity::<GraphBinding, _, _, _, _, _>(
                    JournalActor::reference(),
                )?
                .link_from_path(parent(), ParentJournal::reference(), "parent-audit")
        })
        .unwrap()
        .finish()
        .unwrap()
}
pub fn graph_alias_body() -> ApplicationOperationBody<Schema, GraphAliasSign, SignGraphAlias> {
    let op = GraphAliasSign::reference();
    let next =
        ApplicationOperationBodyValue::<Schema, GraphAliasSign, U64ApplicationValueBinding>::admit(
            "next-revision",
            op.requirement_operands()
                .query_context::<GraphAliasBinding, _, _, _, _>(
                    "context",
                    revision_query::definition(),
                )
                .unwrap(),
            worth_foundational::expression_api::expressions()
                .reading_within(ApplicationOperationBodyExpression::PROFILE)
                .parse("context.revision + uint64(1)")
                .unwrap(),
        )
        .unwrap();
    op.body()
        .in_scope(Row::reference())
        .unwrap()
        .assign_value_at_path(
            parent()
                .forward(ParentRows::reference())
                .reverse(ParentRows::reference()),
            ParentRevision::reference(),
            &next,
        )
        .unwrap()
        .create_in_scope(GraphEvidence::reference(), "verification")
        .unwrap()
        .capture_from_scope(
            RowValue::reference(),
            SignedValue::reference(),
            ApplicationOperationBodyCaptureRole::Subject,
        )
        .unwrap()
        .capture_from_path(
            parent(),
            Company::reference(),
            SignedCompany::reference(),
            ApplicationOperationBodyCaptureRole::Subject,
        )
        .unwrap()
        .capture_from_path(
            parent(),
            ParentRevision::reference(),
            SignedRevision::reference(),
            ApplicationOperationBodyCaptureRole::Subject,
        )
        .unwrap()
        .signed_by::<GraphAliasBinding, _, _, _, _, _, _, _>(
            GraphActor::reference(),
            GraphProvenance::reference(),
            "path-verification",
        )
        .unwrap()
        .link_from_scope(GraphHistory::reference(), "history")
        .unwrap()
        .finish()
        .unwrap()
}
pub fn scope_alias_body() -> ApplicationOperationBody<Schema, ScopeAliasSign, SignScopeAlias> {
    let op = ScopeAliasSign::reference();
    let next =
        ApplicationOperationBodyValue::<Schema, ScopeAliasSign, U64ApplicationValueBinding>::admit(
            "next-revision",
            op.requirement_operands()
                .query_context::<ScopeAliasBinding, _, _, _, _>(
                    "context",
                    revision_query::definition(),
                )
                .unwrap(),
            worth_foundational::expression_api::expressions()
                .reading_within(ApplicationOperationBodyExpression::PROFILE)
                .parse("context.revision + uint64(1)")
                .unwrap(),
        )
        .unwrap();
    op.body()
        .in_scope(Row::reference())
        .unwrap()
        .assign_value_at_path(
            parent().forward(ParentRows::reference()),
            RowValue::reference(),
            &next,
        )
        .unwrap()
        .create_in_scope(ScopeEvidence::reference(), "scope-verification")
        .unwrap()
        .capture_from_scope(
            RowValue::reference(),
            ScopeValue::reference(),
            ApplicationOperationBodyCaptureRole::Subject,
        )
        .unwrap()
        .signed_by::<ScopeAliasBinding, _, _, _, _, _, _, _>(
            ScopeActor::reference(),
            ScopeProvenance::reference(),
            "path-verification",
        )
        .unwrap()
        .link_from_scope(ScopeHistory::reference(), "history")
        .unwrap()
        .finish()
        .unwrap()
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    super::query::declare(super::schema::declare(schema))
        .operation(
            GraphSign::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(graph_body())
                .unwrap(),
        )
        .operation(
            GraphAliasSign::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(graph_alias_body())
                .unwrap(),
        )
        .operation(
            ScopeAliasSign::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(scope_alias_body())
                .unwrap(),
        )
        .operation_read_field(GraphSign::reference(), RowIdentity::reference())
        .operation_read_field(GraphAliasSign::reference(), RowIdentity::reference())
        .operation_read_field(ScopeAliasSign::reference(), RowIdentity::reference())
        .operation_projection_work_budget(GraphSign::reference(), 512)
        .operation_projection_work_budget(GraphAliasSign::reference(), 512)
        .operation_projection_work_budget(ScopeAliasSign::reference(), 512)
        .application_mutation_binding::<GraphBinding>()
        .application_mutation_binding::<GraphAliasBinding>()
        .application_mutation_binding::<ScopeAliasBinding>()
}
