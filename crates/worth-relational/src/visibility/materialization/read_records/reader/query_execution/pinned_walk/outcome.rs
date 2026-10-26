//! Lean visited facts and the reservation that remains with their backing.
use crate::execution::RelationalExecutionDenialCause;
use crate::visibility::materialization::read_records::projection::{
    RelationalEntityMetadata, RelationalRelationMetadata,
};
use worth_execution::ExecutionMemoryReservation;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Actual acquired lists and metadata, independent of private work and request cost.
pub struct PinnedStructuralWalkReads {
    pub adjacency_lists: usize,
    pub relation_records: usize,
    pub entity_records: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A live entity acquired from the exact pinned root, with minimum root distance.
pub struct PinnedStructuralWalkVisit {
    pub entity: RelationalEntityMetadata,
    pub distance: usize,
}

/// Complete canonical structural facts, retaining their original allocation backing.
pub struct PinnedStructuralWalk {
    pub(super) visits: Vec<PinnedStructuralWalkVisit>,
    pub(super) relations: Vec<RelationalRelationMetadata>,
    pub(super) reads: PinnedStructuralWalkReads,
    pub(super) work: usize,
    // The vectors must drop before their original reservation.
    pub(super) _retention: Option<ExecutionMemoryReservation>,
}

impl PinnedStructuralWalk {
    pub fn visits(&self) -> &[PinnedStructuralWalkVisit] {
        &self.visits
    }

    pub fn relations(&self) -> &[RelationalRelationMetadata] {
        &self.relations
    }

    pub const fn reads(&self) -> PinnedStructuralWalkReads {
        self.reads
    }

    pub const fn work_units(&self) -> usize {
        self.work
    }
}

impl std::fmt::Debug for PinnedStructuralWalk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PinnedStructuralWalk")
            .field("visits", &self.visits)
            .field("relations", &self.relations)
            .field("reads", &self.reads)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A complete walk returns no facts when any of these conditions refuses it.
pub enum PinnedStructuralWalkDenialKind {
    Read(RelationalExecutionDenialCause),
    KindUnavailable,
    EndpointUnavailable,
    WrongEntityKind,
    WorkLimitExceeded,
    DistanceOverflow,
}

impl From<RelationalExecutionDenialCause> for PinnedStructuralWalkDenialKind {
    fn from(cause: RelationalExecutionDenialCause) -> Self {
        Self::Read(cause)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// The original refusal with observations and private work already performed.
pub struct PinnedStructuralWalkDenial {
    pub kind: PinnedStructuralWalkDenialKind,
    pub reads: PinnedStructuralWalkReads,
    pub work: usize,
}

impl std::fmt::Display for PinnedStructuralWalkDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pinned structural walk refused: {:?}", self.kind)
    }
}

impl std::error::Error for PinnedStructuralWalkDenial {}
