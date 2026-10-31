//! The same finite endpoint shape serves ordinary Source and managed Parent reads.
use super::super::{
    bounds::{Coordinates, CoordinatesBinding, FixtureLength},
    schema::*,
};
use super::{provider::BoxIndex, schema::*};
use worth_query_decl::facade::{
    application_query::*, application_schema::*, portable_identity::WorthQueryPortableType, *,
};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub coordinate: (u32, u64, u32),
    pub identity: String,
    pub start: Coordinates,
    pub end: Coordinates,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "maintained-geometry.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "maintained-geometry.edge-row.v1" });
worth_query_application_query!(pub SourceQuery for Catalog, identity "maintained-geometry.source.v1", parameters ParametersBinding, result ResultBinding, scope Directory => "Directory", name "geometry_edge_source");
worth_query_application_query!(pub BoxQuery for Catalog, identity "maintained-geometry.box-query.v1", parameters ParametersBinding, result ResultBinding, scope Directory => "Directory", name "geometry_edge_boxes");

pub struct IdentitySlot;
pub struct StartSlot;
pub struct EndSlot;
pub struct StartRegionSlot;
pub struct EndRegionSlot;
pub struct ProbeParameter;
worth_query_portable_type!(IdentitySlot => "maintained-geometry.identity-slot.v1");
worth_query_portable_type!(StartSlot => "maintained-geometry.start-slot.v1");
worth_query_portable_type!(EndSlot => "maintained-geometry.end-slot.v1");
worth_query_portable_type!(StartRegionSlot => "maintained-geometry.start-region-slot.v1");
worth_query_portable_type!(EndRegionSlot => "maintained-geometry.end-region-slot.v1");

pub(super) fn identity<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    IdentitySlot,
    Catalog,
    Edge,
    EdgeFacts,
    EdgeIdentity,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", EdgeIdentity::reference())
}
type RegionSelector<Q, Slot> = ApplicationQueryResultFieldRef<
    Q,
    Slot,
    Catalog,
    Vertex,
    VertexFacts,
    VertexRegion,
    Coordinates,
    ReadWrite,
    EqualityPredicate,
    DeclaredApplicationUnit<FixtureLength, ()>,
>;
fn region<Q, Slot: WorthQueryPortableType>() -> RegionSelector<Q, Slot> {
    ApplicationQueryResultFieldRef::new("region", VertexRegion::reference())
}
pub fn start<Q>() -> ApplicationQueryResultRelationRef<
    Q,
    StartSlot,
    Catalog,
    Starts,
    Edge,
    Vertex,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("start", Starts::reference())
}
pub fn end<Q>() -> ApplicationQueryResultRelationRef<
    Q,
    EndSlot,
    Catalog,
    Ends,
    Edge,
    Vertex,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("end", Ends::reference())
}
pub fn start_region<Q>() -> RegionSelector<Q, StartRegionSlot> {
    region()
}
pub fn end_region<Q>() -> RegionSelector<Q, EndRegionSlot> {
    region()
}

// compiled-example: begin
fn shape<Q: ApplicationQueryMarkerIdentity<Catalog>>(
) -> TypedApplicationQueryResultShape<Catalog, Q, Edge, Row, ResultBinding> {
    ApplicationQueryResultShapeBuilder::new(Edge::reference())
        .field(identity())
        .relation(
            start(),
            ApplicationQueryResultShapeBuilder::<Catalog, Q, Vertex, Row, ResultBinding>::new(
                Vertex::reference(),
            )
            .field(start_region()),
        )
        .relation(
            end(),
            ApplicationQueryResultShapeBuilder::<Catalog, Q, Vertex, Row, ResultBinding>::new(
                Vertex::reference(),
            )
            .field(end_region()),
        )
        .build()
}
// compiled-example: end

pub(super) fn base<Q: ApplicationQueryMarkerIdentity<Catalog, ResultBinding = ResultBinding>>(
    reference: ApplicationQueryReference<Catalog, Q, Parameters, Row, Directory>,
    lanes: ApplicationQueryLaneEligibility,
) -> ApplicationQueryDefinitionBuilder<Catalog, Q, Parameters, Row, Directory> {
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Edge::reference())
        .scope(Directory::reference())
        .result_shape(shape())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(lanes)
        .public()
}
pub fn source_definition(
) -> ApplicationQueryDefinition<Catalog, SourceQuery, Parameters, Row, Directory> {
    base(
        SourceQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot(),
    )
    .root_path(
        ApplicationQueryRootPath::from(Directory::reference()).forward(ContainsEdge::reference()),
    )
    .order_by(identity(), ApplicationQueryOrderingDirection::Descending)
    .build()
    .unwrap()
}
fn probe() -> ApplicationQueryParameterRef<BoxQuery, ProbeParameter, CoordinatesBinding> {
    ApplicationQueryParameterRef::from_query_identifier("probe")
}
pub fn box_definition() -> ApplicationQueryDefinition<Catalog, BoxQuery, Parameters, Row, Directory>
{
    base(
        BoxQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot().with_live(),
    )
    .managed_index(BoxIndex::reference(), probe())
    .order_by(identity(), ApplicationQueryOrderingDirection::Ascending)
    .build()
    .unwrap()
}
impl<Q: ApplicationQueryMarkerIdentity<Catalog>> WorthQueryApplicationProjection<Catalog, Q>
    for Row
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Catalog, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let entity = row.entity_id();
        Ok(Self {
            coordinate: (
                entity.partition_value(),
                entity.local_slot_value(),
                entity.generation_value(),
            ),
            identity: row.field(identity())?,
            start: row.one(start())?.field(start_region())?,
            end: row.one(end())?.field(end_region())?,
        })
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadSource {
    pub directory: String,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindBoxes {
    pub directory: String,
    pub probe: Coordinates,
}
worth_query_structured_value_binding!(pub SourceInputBinding for ReadSource { identity: "maintained-geometry.source-input.v1" });
worth_query_structured_value_binding!(pub BoxInputBinding for FindBoxes { identity: "maintained-geometry.box-input.v1" });
fn source_scope(input: ReadSource) -> String {
    input.directory
}
fn box_scope(input: FindBoxes) -> String {
    input.directory
}
worth_query_query_binding!(pub SourceBinding for ReadSource, schema Catalog, identity "maintained-geometry.source-binding.v1", input SourceInputBinding,
    query SourceQuery, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result ResultBinding,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit, field DirectoryIdentity::reference(), value source_scope, limits results 100000);
worth_query_query_binding!(pub BoxBinding for FindBoxes, schema Catalog, identity "maintained-geometry.box-binding.v1", input BoxInputBinding,
    query BoxQuery, parameters ParametersBinding => |input:&FindBoxes| ApplicationQueryParameterSet::new().bind(probe(),input.probe).unwrap(), result ResultBinding,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit, field DirectoryIdentity::reference(), value box_scope, limits results 4096);
