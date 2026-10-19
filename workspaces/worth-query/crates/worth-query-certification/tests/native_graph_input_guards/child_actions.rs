//! One native candidate owns row creation, repeated-kind cells, nested detail and checked revision.
use super::{
    actions::{DenialBinding, ResultBinding},
    reuse_note_codec::NoteCodec,
    schema::*,
};
use serde::Serialize;
use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs,
    application_query::ApplicationQueryRootPath, application_schema::*, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_mutation_binding, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_relation,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision;
worth_query_entity!(pub Cell for Schema);
worth_query_entity!(pub Detail for Schema);
worth_query_aspect!(pub CellFacts for Schema,Cell;identity=AspectIdentity(0x9175_F101),revision=AspectContractRevision(1),);
worth_query_aspect!(pub DetailFacts for Schema,Detail;identity=AspectIdentity(0x9175_F102),revision=AspectContractRevision(1),);
worth_query_field!(pub CellIdentity for Schema,Cell,CellFacts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub CellKind for Schema,Cell,CellFacts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub CellNote for Schema,Cell,CellFacts:optional String=>NoteCodec,read_only,equality);
worth_query_field!(pub DetailLabel for Schema,Detail,DetailFacts:String=>StringApplicationValueBinding,read_only,equality);
const fn owned() -> ApplicationRelationIntegrity {
    let base = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    ApplicationRelationIntegrity::new(
        base.endpoints,
        ApplicationRelationCardinality::new(None, None, None, Some(1), None, None),
        base.deletion,
    )
}
worth_query_relation!(pub RowCells in Schema,Row=>Cell;integrity=owned());
worth_query_relation!(pub RowCellArchive in Schema,Row=>Cell;integrity=owned());
worth_query_relation!(pub CellDetails in Schema,Cell=>Detail;integrity=owned());
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
worth_query_operation_reads!(Create=>[Parent,ParentIdentity,ParentRevision]);
worth_query_operation_writes!(Create=>[ParentRevision,RowIdentity,RowValue,CellIdentity,CellKind,CellNote,DetailLabel]);
worth_query_operation_creates!(Create=>[Row,Cell,Detail]);
worth_query_operation_links!(Create=>[ParentRows,RowCells,RowCellArchive,CellDetails]);
fn scope(input: &CreateTree) -> String {
    input.parent.clone()
}
worth_query_mutation_binding!(pub Binding for CreateTree,schema Schema,
    identity "worth.query.child.binding.v1",input Input,operation Create,result ResultBinding,
    idempotency u64,identity "worth.query.child.request.v1",decision WorthQueryOperationBodyDecision<Schema,Parent>,denial DenialBinding,
    handler identity "worth.query.child.body.v1",program required,outputs NoApplicationMutationOutputs,
    principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Parent,ParentFacts,ParentIdentity,String,ReadOnly,NoApplicationUnit,field ParentIdentity::reference(),value scope,
    candidates creates 4,deletes 0,links 6,unlinks 0,writes 10,emits 0,resources retained_representation_bytes 16384);
pub fn body() -> ApplicationOperationBody<Schema, Create, CreateTree> {
    let op = Create::reference();
    let next = ApplicationOperationBodyValue::<Schema, Create, U64ApplicationValueBinding>::admit(
        "next",
        op.requirement_operands()
            .field("revision", ParentRevision::reference())
            .unwrap(),
        worth_foundational::expression_api::expressions()
            .reading_within(ApplicationOperationBodyExpression::PROFILE)
            .parse("revision + uint64(1)")
            .unwrap(),
    )
    .unwrap();
    op.body()
        .assign_value(ParentRevision::reference(), &next)
        .unwrap()
        .expect_input_from_path(
            ApplicationQueryRootPath::from(Parent::reference()),
            ParentIdentity::reference(),
            "parent",
        )
        .unwrap()
        .create_in_scope(Row::reference(), "row")
        .unwrap()
        .initialize_input(RowIdentity::reference(), "row")
        .unwrap()
        .initialize_value(RowValue::reference(), &next)
        .unwrap()
        .link_from_scope(ParentRows::reference(), "parent-row")
        .unwrap()
        .with_child(RowCells::reference(), "physical", |cell| {
            cell.initialize_input(CellIdentity::reference(), "physical")?
                .initialize(CellKind::reference(), 1)?
                .initialize_optional_input(CellNote::reference(), "physical_note")?
                .link_from_parent(RowCellArchive::reference(), "physical-archive")?
                .with_child(CellDetails::reference(), "detail", |detail| {
                    detail.initialize_input(DetailLabel::reference(), "detail")
                })
        })
        .unwrap()
        .with_child(RowCells::reference(), "electronic", |cell| {
            cell.initialize_input(CellIdentity::reference(), "electronic")?
                .initialize(CellKind::reference(), 2)?
                .initialize_optional_input(CellNote::reference(), "electronic_note")?
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
        .entity(Cell::reference())
        .entity(Detail::reference())
        .aspect(Cell::reference(), CellFacts::reference())
        .aspect(Detail::reference(), DetailFacts::reference())
        .field(Cell::reference(), CellIdentity::reference())
        .field(Cell::reference(), CellKind::reference())
        .field(Cell::reference(), CellNote::reference())
        .field(Detail::reference(), DetailLabel::reference())
        .relation(RowCells::reference(), Row::reference(), Cell::reference())
        .relation(
            RowCellArchive::reference(),
            Row::reference(),
            Cell::reference(),
        )
        .relation(
            CellDetails::reference(),
            Cell::reference(),
            Detail::reference(),
        )
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
