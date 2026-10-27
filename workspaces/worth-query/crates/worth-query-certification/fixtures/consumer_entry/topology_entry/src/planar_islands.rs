//! Authored coupling joins rings into islands; successor still owns ring order.
use super::{Body, PlanarSuccessor, TopologyModelBinding};
use worth_query_decl::facade::application_schema::*;

pub struct PlanarCoupling;
impl PlanarCoupling {
    pub const fn reference<S: TopologyModelBinding>() -> ApplicationRelationRef<S, Self, Body, Body>
    {
        ApplicationRelationRef::from_schema_identifiers(
            "PlanarCoupling",
            "Body",
            "Body",
            ApplicationRelationIntegrity::new(
                ApplicationRelationEndpoints::same_context(false),
                ApplicationRelationCardinality::new(None, None, None, None, None, Some(1)),
                ApplicationRelationDeletionPolicy::CascadeDeleteRelations,
            ),
        )
    }
}

pub struct PlanarIslands;
impl PlanarIslands {
    pub const fn reference<S: TopologyModelBinding>() -> ApplicationComponentSetRef<S, Self, Body> {
        ApplicationComponentSetRef::new("PlanarIslands", Body::reference())
    }
}

pub(super) fn declare<S: TopologyModelBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .relation(
            PlanarCoupling::reference(),
            Body::reference(),
            Body::reference(),
        )
        .components(
            ApplicationComponentSetDefinition::new(PlanarIslands::reference())
                .coupled_by(PlanarSuccessor::reference())
                .coupled_by(PlanarCoupling::reference()),
        )
}
