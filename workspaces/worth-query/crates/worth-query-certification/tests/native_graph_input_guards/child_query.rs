//! Governed projection proves distinct child entities and the actual nested parent relation.
use super::{child_actions::*, schema::*};
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_application_query,
    worth_query_portable_type, worth_query_query_binding, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};
#[derive(Clone)]
pub struct Read(pub String);
#[derive(Clone)]
pub struct ReadArchive(pub String);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value {
    pub id: u64,
    pub kind: u64,
    pub note: Option<String>,
    pub detail: Option<String>,
    pub entity: worth_relational::facade::identity::EntityId,
}
worth_query_portable_type!(Value=>"worth.query.child.cell-result.v1");
worth_query_structured_value_binding!(pub Input for Read {identity:"worth.query.child.read.v1"});
worth_query_structured_value_binding!(pub ArchiveInput for ReadArchive {identity:"worth.query.child.archive-read.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Value {identity:"worth.query.child.cell-result.v1"});
worth_query_structured_value_binding!(pub Parameters for () {identity:"worth.query.child.parameters.v1"});
worth_query_structured_value_binding!(pub DetailResult for String {identity:"worth.query.child.detail-result.v1"});
worth_query_application_query!(pub Query for Schema,identity "ChildQuery",parameters Parameters,result ResultBinding,scope Row=>"Row",name "row_cells");
worth_query_application_query!(pub ArchiveQuery for Schema,identity "ChildArchiveQuery",parameters Parameters,result ResultBinding,scope Row=>"Row",name "row_cell_archive");
macro_rules! field {
    ($slot:ident,$name:ident,$entity:ty,$aspect:ty,$field:ty,$value:ty,$identity:literal)=>{
        pub struct $slot;
        worth_query_portable_type!($slot=>$identity);
        fn $name<Q:ApplicationQueryMarkerIdentity<Schema>>()->ApplicationQueryResultFieldRef<Q,$slot,Schema,$entity,$aspect,$field,$value,ReadOnly,EqualityPredicate,NoApplicationUnit>{ApplicationQueryResultFieldRef::new(stringify!($name),<$field>::reference())}
    };
}
field!(
    Id,
    id,
    Cell,
    CellFacts,
    CellIdentity,
    u64,
    "worth.query.child.id-slot.v1"
);
field!(
    Kind,
    kind,
    Cell,
    CellFacts,
    CellKind,
    u64,
    "worth.query.child.kind-slot.v1"
);
field!(
    Label,
    label,
    Detail,
    DetailFacts,
    DetailLabel,
    String,
    "worth.query.child.label-slot.v1"
);
pub struct Note;
worth_query_portable_type!(Note=>"worth.query.child.note-slot.v1");
fn note<Q: ApplicationQueryMarkerIdentity<Schema>>() -> ApplicationQueryOptionalResultFieldRef<
    Q,
    Note,
    Schema,
    Cell,
    CellFacts,
    CellNote,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryOptionalResultFieldRef::new("note", CellNote::reference())
}
pub struct Details;
worth_query_portable_type!(Details=>"worth.query.child.details-slot.v1");
fn details<Q: ApplicationQueryMarkerIdentity<Schema>>() -> ApplicationQueryResultRelationRef<
    Q,
    Details,
    Schema,
    CellDetails,
    Cell,
    Detail,
    ForwardResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::forward_optional("details", CellDetails::reference())
}
fn shape<Q: ApplicationQueryMarkerIdentity<Schema>>(
) -> ApplicationQueryResultShapeBuilder<Schema, Q, Cell, Value, ResultBinding> {
    let nested =
        ApplicationQueryResultShapeBuilder::<Schema, Q, Detail, String, DetailResult>::new(
            Detail::reference(),
        )
        .field(label());
    ApplicationQueryResultShapeBuilder::<Schema, Q, Cell, Value, ResultBinding>::new(
        Cell::reference(),
    )
    .field(id())
    .field(kind())
    .optional_field(note())
    .relation(details(), nested)
}
pub fn definition() -> ApplicationQueryDefinition<Schema, Query, (), Value, Row> {
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Cell::reference())
        .scope(Row::reference())
        .result_shape(shape().build())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(ApplicationQueryRootPath::from(Row::reference()).forward(RowCells::reference()))
        .build()
        .unwrap()
}
pub fn archive_definition() -> ApplicationQueryDefinition<Schema, ArchiveQuery, (), Value, Row> {
    ApplicationQueryDefinitionBuilder::declare(ArchiveQuery::reference())
        .root(Cell::reference())
        .scope(Row::reference())
        .result_shape(shape().build())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Row::reference()).forward(RowCellArchive::reference()),
        )
        .build()
        .unwrap()
}
impl<Q: ApplicationQueryMarkerIdentity<Schema>> WorthQueryApplicationProjection<Schema, Q>
    for Value
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            id: row.field(id())?,
            kind: row.field(kind())?,
            note: row.optional_field(note())?,
            detail: row
                .optional(details())?
                .map(|detail| detail.field(label()))
                .transpose()?,
            entity: row.entity_id(),
        })
    }
}
fn scope(read: Read) -> String {
    read.0
}
fn archive_scope(read: ReadArchive) -> String {
    read.0
}
worth_query_query_binding!(pub ReadArchiveBinding for ReadArchive,schema Schema,identity "worth.query.child.archive-read-binding.v1",input ArchiveInput,query ArchiveQuery,parameters Parameters=>|_|ApplicationQueryParameterSet::new(),result ResultBinding,
    principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value archive_scope,limits results 20);
worth_query_query_binding!(pub ReadBinding for Read,schema Schema,identity "worth.query.child.read-binding.v1",input Input,query Query,parameters Parameters=>|_|ApplicationQueryParameterSet::new(),result ResultBinding,
    principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value scope,limits results 20);
