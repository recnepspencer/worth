//! Edges retain real endpoint relations; vertex-region changes own their bounds.
use super::super::{
    bounds::{Coordinates, CoordinatesBinding, FixtureLength},
    schema::{Catalog, Directory},
};
use worth_query_decl::facade::{application_schema::*, *};

worth_query_entity!(pub Vertex for Catalog);
worth_query_entity!(pub Edge for Catalog);
worth_query_aspect!(pub VertexFacts for Catalog, Vertex; identity = AspectIdentity(0x919021), revision = AspectContractRevision(1));
worth_query_aspect!(pub EdgeFacts for Catalog, Edge; identity = AspectIdentity(0x919022), revision = AspectContractRevision(1));
worth_query_field!(pub VertexIdentity for Catalog, Vertex, VertexFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub VertexRegion for Catalog, Vertex, VertexFacts: Coordinates => CoordinatesBinding, unit FixtureLength, read_write, equality);
worth_query_field!(pub EdgeIdentity for Catalog, Edge, EdgeFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_relation!(pub ContainsEdge in Catalog, Directory => Edge; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub Starts in Catalog, Edge => Vertex; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub Ends in Catalog, Edge => Vertex; integrity = same_context_unbounded_retain_dangling);

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    schema
        .entity(Vertex::reference())
        .entity(Edge::reference())
        .aspect(Vertex::reference(), VertexFacts::reference())
        .aspect(Edge::reference(), EdgeFacts::reference())
        .field(Vertex::reference(), VertexIdentity::reference())
        .field(Vertex::reference(), VertexRegion::reference())
        .field(Edge::reference(), EdgeIdentity::reference())
        .relation(
            ContainsEdge::reference(),
            super::super::schema::Directory::reference(),
            Edge::reference(),
        )
        .relation(Starts::reference(), Edge::reference(), Vertex::reference())
        .relation(Ends::reference(), Edge::reference(), Vertex::reference())
}
