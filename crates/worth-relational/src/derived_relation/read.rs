//! Immutable complete derivation facts, separate from source authority.
use crate::identity::data::{EntityId, KindId};
use crate::validation::data::{StructuralRelationRecord, StructuralWalkDenialKind};

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
    pub(super) endpoints: Vec<EntityId>,
    pub(super) entities: Vec<(EntityId, KindId)>,
    pub(super) relations: Vec<StructuralRelationRecord>,
    pub(super) adjacency: Vec<DerivedRelationAdjacency>,
    pub(super) reads: DerivedRelationReads,
}

impl DerivedRelationData {
    pub fn endpoints(&self) -> &[EntityId] {
        &self.endpoints
    }
    pub fn entity_dependencies(&self) -> &[(EntityId, KindId)] {
        &self.entities
    }
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DerivedRelationReadDenial {
    Definition(super::DerivedRelationDenial),
    Structural(StructuralWalkDenialKind),
    Request(crate::execution::RelationalExecutionDenialCause),
}

impl From<super::DerivedRelationDenial> for DerivedRelationReadDenial {
    fn from(denial: super::DerivedRelationDenial) -> Self {
        Self::Definition(denial)
    }
}
