use crate::identity::data::{EntityId, KindId};

use super::super::{CustomInvariantInterruption, StructuralReadError, StructuralRelationRecord};

/// Complete-walk boundary acquisitions: materialized adjacency lists and
/// explicit entity/relation reads, including metadata denied after acquisition.
/// Native visibility/filtering mechanics remain on the native work meter.
/// These observations confer no admission, identity or disclosure authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StructuralWalkReads {
    pub adjacency_lists: usize,
    pub relation_records: usize,
    pub entity_records: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StructuralWalkStep {
    pub entity: EntityId,
    pub kind: KindId,
    pub distance: usize,
}

/// A complete walk. Roots appear at distance zero; visits sort by minimum
/// distance then native identity. Every matching physical edge is retained,
/// including self/back/diamond edges that discover no new entity.
pub struct CompleteStructuralWalk {
    pub(super) steps: Vec<StructuralWalkStep>,
    pub(super) relations: Vec<StructuralRelationRecord>,
    pub(super) reads: StructuralWalkReads,
    // Owned result memory remains on the original native payer after its
    // temporary view and frontier have gone away.
    pub(super) _retention: super::super::CustomInvariantWorkMeter,
}

impl CompleteStructuralWalk {
    pub fn steps(&self) -> &[StructuralWalkStep] {
        &self.steps
    }

    pub fn relations(&self) -> &[StructuralRelationRecord] {
        &self.relations
    }

    pub const fn reads(&self) -> StructuralWalkReads {
        self.reads
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuralWalkDenialKind {
    Interrupted(CustomInvariantInterruption),
    Read(StructuralReadError),
    MemoryBudgetExceeded,
    MemoryAdmission(worth_execution::MemoryLimitDenial),
}

/// A refused walk has observations but never a partial successful result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StructuralWalkDenial {
    pub kind: StructuralWalkDenialKind,
    pub reads: StructuralWalkReads,
}

impl std::fmt::Debug for CompleteStructuralWalk {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CompleteStructuralWalk")
            .field("steps", &self.steps)
            .field("relations", &self.relations)
            .field("reads", &self.reads)
            .finish_non_exhaustive()
    }
}
