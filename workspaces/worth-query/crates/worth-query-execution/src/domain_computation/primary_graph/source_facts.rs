//! Native physical read facts shared by Query and operation selection issuers.
//! These facts describe reads; they do not grant access or mint aggregate identity.
use super::structural_walk::StructuralWalkDependencies;
use std::sync::Arc;
use worth_relational::facade::identity::EntityId;
mod allocation;
mod builder;
mod descriptor;
mod retention;
mod revisions;
pub(in crate::domain_computation::primary_graph) use allocation::SourceFactsAllocation;
pub(in crate::domain_computation::primary_graph) use builder::{
    SourceFactReadDenial, SourceFactsBuilder,
};
pub(in crate::domain_computation::primary_graph) use descriptor::encode_text;
pub(in crate::domain_computation::primary_graph) use revisions::adjacency_direction_rank;
pub(in crate::domain_computation) use revisions::{
    WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct SourceFacts {
    pub(in crate::domain_computation::primary_graph) entities: Vec<EntityId>,
    pub(in crate::domain_computation::primary_graph) aspects: Vec<WorthQueryObservedFieldRevision>,
    pub(in crate::domain_computation::primary_graph) adjacencies:
        Vec<WorthQueryObservedAdjacencyRevision>,
    pub(in crate::domain_computation::primary_graph) walks: Option<Arc<StructuralWalkDependencies>>,
    pub(in crate::domain_computation::primary_graph) allocation: Option<Arc<SourceFactsAllocation>>,
}

impl PartialEq for SourceFacts {
    fn eq(&self, other: &Self) -> bool {
        self.entities == other.entities
            && self.aspects == other.aspects
            && self.adjacencies == other.adjacencies
            && self.walks == other.walks
    }
}

impl Eq for SourceFacts {}
