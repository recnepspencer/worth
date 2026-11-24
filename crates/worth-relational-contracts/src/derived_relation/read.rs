//! Immutable complete derivation facts, separate from source authority.
use crate::graph_member::NativeMemberKey;
use crate::identity::data::{EntityId, KindId};
use crate::validation::data::structural_read::StructuralRelationRecord;

/// Every explored physical adjacency, including an empty one.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DerivedRelationAdjacency {
    pub anchor: EntityId,
    pub relation: KindId,
    pub outgoing: bool,
}

/// Actual source acquisitions; these counters supply no authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DerivedRelationReads {
    pub adjacency_lists: usize,
    pub relation_records: usize,
    pub entity_records: usize,
}

/// A complete endpoint set plus all physical support and negative reads.
#[derive(Debug)]
pub struct DerivedRelationData {
    endpoints: Vec<NativeMemberKey>,
    entities: Vec<(EntityId, KindId)>,
    relations: Vec<StructuralRelationRecord>,
    adjacency: Vec<DerivedRelationAdjacency>,
    reads: DerivedRelationReads,
}

impl DerivedRelationData {
    /// Canonical sorted, distinct member keys from the complete acquisition.
    pub fn endpoints(&self) -> &[NativeMemberKey] {
        &self.endpoints
    }
    pub fn entity_dependencies(&self) -> &[(EntityId, KindId)] {
        &self.entities
    }
    /// Canonical relation-ID order, retaining every distinct physical edge.
    pub fn physical_relations(&self) -> &[StructuralRelationRecord] {
        &self.relations
    }
    pub fn adjacency_dependencies(&self) -> &[DerivedRelationAdjacency] {
        &self.adjacency
    }
    pub const fn reads(&self) -> DerivedRelationReads {
        self.reads
    }
}

/// A refused evaluation never returns partial successful data.
pub use super::read_denial::DerivedRelationReadDenial;

/// Assemble the canonical facts retained by a successful evaluator.
pub fn complete_data(
    endpoints: Vec<NativeMemberKey>,
    entities: Vec<(EntityId, KindId)>,
    relations: Vec<StructuralRelationRecord>,
    adjacency: Vec<DerivedRelationAdjacency>,
    reads: DerivedRelationReads,
) -> DerivedRelationData {
    DerivedRelationData {
        endpoints,
        entities,
        relations,
        adjacency,
        reads,
    }
}

/// Consume support from an already charged evaluation without cloning any roster.
pub fn take_dependencies(
    data: DerivedRelationData,
) -> (
    Vec<(EntityId, KindId)>,
    Vec<StructuralRelationRecord>,
    Vec<DerivedRelationAdjacency>,
) {
    (data.entities, data.relations, data.adjacency)
}
