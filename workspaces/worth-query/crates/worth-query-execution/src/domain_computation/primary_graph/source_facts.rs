//! Native physical read facts shared by Query and operation selection issuers.
//! These facts describe reads; they do not grant access or mint aggregate identity.
use super::derived_source::DerivedSourceDependencies;
use super::structural_walk::StructuralWalkDependencies;
use std::sync::Arc;
use worth_relational::facade::identity::EntityId;
pub mod allocation;
mod authorization_currentness;
pub(in crate::domain_computation::primary_graph) use authorization_currentness::map_source_authorization_currentness;
pub(in crate::domain_computation) use authorization_currentness::validate_source_principal_facts;
mod builder;
mod descriptor;
mod retention;
pub mod revisions;
pub(in crate::domain_computation) use allocation::SourceFactsAllocation;
pub(in crate::domain_computation::primary_graph) use builder::{
    SourceFactReadDenial, SourceFactsBuilder,
};
pub(in crate::domain_computation::primary_graph) use descriptor::encode_text;
pub(in crate::domain_computation::primary_graph) use revisions::adjacency_direction_rank;
pub(in crate::domain_computation) use revisions::{
    WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
};

#[derive(Debug)]
pub struct SourceFacts {
    pub entities: Vec<EntityId>,
    pub aspects: Vec<WorthQueryObservedFieldRevision>,
    pub adjacencies: Vec<WorthQueryObservedAdjacencyRevision>,
    pub walks: Option<Arc<StructuralWalkDependencies>>,
    pub derived: Option<Arc<DerivedSourceDependencies>>,
    pub allocation: Option<Arc<SourceFactsAllocation>>,
}

impl PartialEq for SourceFacts {
    fn eq(&self, other: &Self) -> bool {
        self.entities == other.entities
            && self.aspects == other.aspects
            && self.adjacencies == other.adjacencies
            && self.walks == other.walks
            && self.derived == other.derived
    }
}

impl Eq for SourceFacts {}
