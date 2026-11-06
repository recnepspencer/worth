//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct GeometryRecords;
impl fragment::Fragment for GeometryRecords {
    const NAME: &'static str = "geometry_records";
    type Bindings = GeometryRecordsBindings;
    type Members = (Vertex, Edge);
}
pub struct GeometryRecordsBindings;
impl fragment::BindingSet for GeometryRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<String> for GeometryRecordsBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<Coordinates> for GeometryRecordsBindings {
    type Binding = CoordinatesBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
pub struct Vertex;
impl fragment::Entity for Vertex {
    const NAME: &'static str = "Vertex";
    type Aspects = (VertexFacts,);
    type Relations = ();
}
impl Vertex {
    pub fn reference() -> ApplicationEntityRef<Catalog, Self> {
        entity(GeometryRecords, Self)
    }
}
pub struct Edge;
impl fragment::Entity for Edge {
    const NAME: &'static str = "Edge";
    type Aspects = (EdgeFacts,);
    type Relations = (Starts, Ends);
}
impl Edge {
    pub fn reference() -> ApplicationEntityRef<Catalog, Self> {
        entity(GeometryRecords, Self)
    }
}
pub struct VertexFacts;
impl fragment::Aspect for VertexFacts {
    const NAME: &'static str = "VertexFacts";
    type Row = VertexFactsValue;
}

pub struct VertexFactsValue {
    pub vertex_identity: String,
    pub vertex_region: Coordinates,
}
impl fragment::Record<VertexFactsValue> for GeometryRecordsBindings {
    fn record() -> impl fragment::Shape<VertexFactsValue, Self> {
        fragment::field("VertexIdentity", |row: &VertexFactsValue| {
            row.vertex_identity.clone()
        })
        .and("VertexRegion", |row: &VertexFactsValue| row.vertex_region)
        .build(
            |vertex_identity, vertex_region, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(VertexFactsValue {
                    vertex_identity,
                    vertex_region,
                })
            },
        )
    }
}
pub struct EdgeFacts;
impl fragment::Aspect for EdgeFacts {
    const NAME: &'static str = "EdgeFacts";
    type Row = EdgeFactsValue;
}

pub struct EdgeFactsValue {
    pub edge_identity: String,
}
impl fragment::Record<EdgeFactsValue> for GeometryRecordsBindings {
    fn record() -> impl fragment::Shape<EdgeFactsValue, Self> {
        fragment::field("EdgeIdentity", |row: &EdgeFactsValue| {
            row.edge_identity.clone()
        })
        .build(|edge_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(EdgeFactsValue { edge_identity })
        })
    }
}
pub type VertexIdentity = FieldSelection<GeometryRecords, Vertex, VertexFacts, String>;
pub type VertexRegion = FieldSelection<GeometryRecords, Vertex, VertexFacts, Coordinates>;
pub type EdgeIdentity = FieldSelection<GeometryRecords, Edge, EdgeFacts, String>;
pub struct ContainsEdge;
impl fragment::Relation for ContainsEdge {
    const NAME: &'static str = "ContainsEdge";
    type To = Edge;
}
impl ContainsEdge {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, Directory, Edge> {
        ApplicationRelationRef::from_schema_identifiers(
            <Self as Relation>::NAME,
            <Directory as Entity>::NAME,
            <Edge as Entity>::NAME,
            relation_integrity::<Self>(),
        )
    }
}
pub struct Starts;
impl fragment::Relation for Starts {
    const NAME: &'static str = "Starts";
    type To = Vertex;
}
impl Starts {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, Edge, Vertex> {
        relation(GeometryRecords, Edge, Self)
    }
}
pub struct Ends;
impl fragment::Relation for Ends {
    const NAME: &'static str = "Ends";
    type To = Vertex;
}
impl Ends {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, Edge, Vertex> {
        relation(GeometryRecords, Edge, Self)
    }
}
