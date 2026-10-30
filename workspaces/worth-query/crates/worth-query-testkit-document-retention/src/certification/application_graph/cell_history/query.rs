//! Ordinary live cells are the actual outgoing members at the selected basis.
use super::schema::*;
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

pub struct Parameters;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellRow {
    pub key: u64,
    pub value: u64,
    pub occurrence: (u32, u64, u32),
}
worth_query_structured_value_binding!(pub ParametersBinding for Parameters {identity:"cell-history.query.parameters.v1"});
worth_query_structured_value_binding!(pub RowBinding for CellRow {identity:"cell-history.query.row.v1"});
worth_query_application_query!(pub Cells for Schema, identity "cell-history.query.v1",
    parameters ParametersBinding, result RowBinding, scope Feature => "Feature", name "live_cells");
pub struct KeySlot;
pub struct ValueSlot;
worth_query_portable_type!(KeySlot => "cell-history.key-slot.v1");
worth_query_portable_type!(ValueSlot => "cell-history.value-slot.v1");
fn key() -> ApplicationQueryResultFieldRef<
    Cells,
    KeySlot,
    Schema,
    Cell,
    CellFacts,
    CellKey,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("key", CellKey::reference())
}
fn value() -> ApplicationQueryResultFieldRef<
    Cells,
    ValueSlot,
    Schema,
    Cell,
    CellFacts,
    CellValue,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", CellValue::reference())
}
impl WorthQueryApplicationProjection<Schema, Cells> for CellRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Cells>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let entity = row.entity_id();
        Ok(Self {
            key: row.field(key())?,
            value: row.field(value())?,
            occurrence: (
                entity.partition_value(),
                entity.local_slot_value(),
                entity.generation_value(),
            ),
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
pub struct LiveCells {
    pub feature: u64,
}
worth_query_structured_value_binding!(pub ReadInputBinding for LiveCells {identity:"cell-history.query.input.v1"});
fn feature(input: LiveCells) -> u64 {
    input.feature
}
worth_query_query_binding!(pub ReadBinding for LiveCells, schema Schema,
    identity "cell-history.query.binding.v1", input ReadInputBinding,
    query Cells, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result RowBinding,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Feature,FeatureFacts,FeatureKey,u64,ReadOnly,NoApplicationUnit,
    field FeatureKey::reference(), value feature, limits results 16_384);

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let shape =
        ApplicationQueryResultShapeBuilder::<Schema, Cells, Cell, CellRow, RowBinding>::new(
            Cell::reference(),
        )
        .field(key())
        .field(value())
        .build();
    let query = ApplicationQueryDefinitionBuilder::declare(Cells::reference())
        .root(Cell::reference())
        .scope(Feature::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Feature::reference()).forward(OwnsCell::reference()),
        )
        .build()
        .unwrap();
    schema
        .application_query(query)
        .application_query_binding::<ReadBinding>()
}
