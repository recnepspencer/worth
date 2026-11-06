//! Authored coupling joins rings into islands; successor still owns ring order.
use super::{Body, PlanarSuccessor, TopologyModelBinding, TopologyRecords};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::application_schema::{
    fragment, ApplicationRelationCardinality, ApplicationRelationDeletionPolicy,
};

pub struct PlanarCoupling;
impl fragment::Relation for PlanarCoupling {
    const NAME: &'static str = "PlanarCoupling";
    type To = Body;
    const SELF_EDGES: bool = false;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, None, None, None, None, Some(1));
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::CascadeDeleteRelations;
}
impl PlanarCoupling {
    pub fn reference<S: TopologyModelBinding>() -> ApplicationRelationRef<S, Self, Body, Body> {
        fragment::relation(TopologyRecords, Body, Self)
    }
}

pub struct PlanarIslands;
impl PlanarIslands {
    pub fn reference<S: TopologyModelBinding>() -> ApplicationComponentSetRef<S, Self, Body> {
        ApplicationComponentSetRef::new("PlanarIslands", Body::reference())
    }
}

pub(super) fn declare<S: TopologyModelBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema.components(
        ApplicationComponentSetDefinition::new(PlanarIslands::reference())
            .coupled_by(PlanarSuccessor::reference())
            .coupled_by(PlanarCoupling::reference()),
    )
}
