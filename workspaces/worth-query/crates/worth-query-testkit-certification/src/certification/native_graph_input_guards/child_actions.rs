//! One native candidate owns row creation, repeated-kind cells, nested detail and checked revision.
pub use super::{
    actions::{DenialBinding, ResultBinding},
    reuse_note_codec::NoteCodec,
    schema::*,
};
pub use serde::Serialize;
pub use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs,
    application_query::ApplicationQueryRootPath, application_schema::*,
    worth_query_mutation_binding, worth_query_operation, worth_query_operation_creates,
    worth_query_operation_links, worth_query_operation_reads, worth_query_operation_writes,
};
pub use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision;

#[derive(Clone, Serialize)]
pub struct CreateTree {
    pub parent: String,
    pub row: String,
    pub physical: u64,
    pub electronic: u64,
    pub physical_note: Option<String>,
    pub electronic_note: Option<String>,
    pub detail: String,
}
pub struct Input;
impl ApplicationStructuredValueBinding for Input {
    type Value = CreateTree;
    const IDENTITY_NAME: &'static str = "worth.query.child.input.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &CreateTree) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
worth_query_operation!(pub Create for Schema,input Input);
worth_query_operation_reads!(Create => [Parent]);

worth_query_operation_creates!(Create=>[Row,Cell,Detail]);
worth_query_operation_links!(Create=>[ParentRows,RowCells,RowCellArchive,CellDetails]);
pub fn scope(input: &CreateTree) -> String {
    input.parent.clone()
}
worth_query_mutation_binding!(pub Binding for CreateTree,schema Schema,
identity "worth.query.child.binding.v1",input Input,operation Create,result ResultBinding,
idempotency u64,identity "worth.query.child.request.v1",decision WorthQueryOperationBodyDecision<Schema,Parent>,denial DenialBinding,
handler identity "worth.query.child.body.v1",program required,outputs NoApplicationMutationOutputs,
principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
scope ParentIdentity, field ParentIdentity::at("ParentIdentity"),value scope,
);
pub fn body() -> ApplicationOperationBody<Schema, Create, CreateTree> {
    let op = Create::reference();
    let next = ApplicationOperationBodyValue::<Schema, Create, U64ApplicationValueBinding>::admit(
        "next",
        op.requirement_operands()
            .field("revision", ParentRevision::at("ParentRevision"))
            .unwrap(),
        worth_foundational::expression_api::expressions()
            .reading_within(ApplicationOperationBodyExpression::PROFILE)
            .parse("revision + uint64(1)")
            .unwrap(),
    )
    .unwrap();
    op.body()
        .assign_value(ParentRevision::at("ParentRevision"), &next)
        .unwrap()
        .expect_input_from_path(
            ApplicationQueryRootPath::from(Parent::reference()),
            ParentIdentity::at("ParentIdentity"),
            "parent",
        )
        .unwrap()
        .create_in_scope(Row::reference(), "row")
        .unwrap()
        .initialize_input(RowIdentity::at("RowIdentity"), "row")
        .unwrap()
        .initialize_value(RowValue::at("RowValue"), &next)
        .unwrap()
        .link_from_scope(ParentRows::reference(), "parent-row")
        .unwrap()
        .with_child(RowCells::reference(), "physical", |cell| {
            cell.initialize_input(CellIdentity::at("CellIdentity"), "physical")?
                .initialize(CellKind::at("CellKind"), 1)?
                .initialize_optional_input(CellNote::at("CellNote"), "physical_note")?
                .link_from_parent(RowCellArchive::reference(), "physical-archive")?
                .with_child(CellDetails::reference(), "detail", |detail| {
                    detail.initialize_input(DetailLabel::at("DetailLabel"), "detail")
                })
        })
        .unwrap()
        .with_child(RowCells::reference(), "electronic", |cell| {
            cell.initialize_input(CellIdentity::at("CellIdentity"), "electronic")?
                .initialize(CellKind::at("CellKind"), 2)?
                .initialize_optional_input(CellNote::at("CellNote"), "electronic_note")?
                .link_from_parent(RowCellArchive::reference(), "electronic-archive")
        })
        .unwrap()
        .finish()
        .unwrap()
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .operation(
            Create::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(body())
                .unwrap(),
        )
        .operation_projection_work_budget(Create::reference(), 256)
        .application_mutation_binding::<Binding>()
}

pub use crate::facade::certification::native_graph_input_guards::schema::storage_model::{
    Cell, CellDetails, CellFacts, CellIdentity, CellKind, CellNote, Detail, DetailFacts,
    DetailLabel, RowCellArchive, RowCells,
};
