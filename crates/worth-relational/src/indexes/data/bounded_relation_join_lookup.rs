use crate::identity::data::EntityId;
use crate::snapshots::data::SnapshotHandle;

use super::{
    BoundedIndexParityMode, DerivedIndexGenerationId, DerivedIndexId, MAX_BOUNDED_INDEX_CANDIDATES,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedRelationJoinLookupRequest {
    snapshot: SnapshotHandle,
    index_id: DerivedIndexId,
    left_entity_id: EntityId,
    right_entity_id: EntityId,
    candidate_limit: usize,
}

impl BoundedRelationJoinLookupRequest {
    pub fn new(
        snapshot: SnapshotHandle,
        index_id: DerivedIndexId,
        left_entity_id: EntityId,
        right_entity_id: EntityId,
        candidate_limit: usize,
    ) -> Result<Self, BoundedRelationJoinLookupDenial> {
        if candidate_limit == 0 || candidate_limit > MAX_BOUNDED_INDEX_CANDIDATES {
            return Err(BoundedRelationJoinLookupDenial::new(
                BoundedRelationJoinLookupDenialKind::InvalidCandidateLimit,
                index_id,
            ));
        }
        Ok(Self {
            snapshot,
            index_id,
            left_entity_id,
            right_entity_id,
            candidate_limit,
        })
    }

    pub fn snapshot(&self) -> &SnapshotHandle {
        &self.snapshot
    }

    pub const fn index_id(&self) -> DerivedIndexId {
        self.index_id
    }

    pub const fn left_entity_id(&self) -> EntityId {
        self.left_entity_id
    }

    pub const fn right_entity_id(&self) -> EntityId {
        self.right_entity_id
    }

    pub const fn candidate_limit(&self) -> usize {
        self.candidate_limit
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedRelationJoinLookupOutcome {
    generation_id: DerivedIndexGenerationId,
    candidate_entity_ids: Vec<EntityId>,
    work: BoundedRelationJoinLookupWork,
    overflowed: bool,
    parity_mode: BoundedIndexParityMode,
}

impl BoundedRelationJoinLookupOutcome {
    pub(crate) fn new(
        generation_id: DerivedIndexGenerationId,
        candidate_entity_ids: Vec<EntityId>,
        work: BoundedRelationJoinLookupWork,
        overflowed: bool,
        parity_mode: BoundedIndexParityMode,
    ) -> Self {
        Self {
            generation_id,
            candidate_entity_ids,
            work,
            overflowed,
            parity_mode,
        }
    }

    pub const fn generation_id(&self) -> DerivedIndexGenerationId {
        self.generation_id
    }

    pub fn candidate_entity_ids(&self) -> &[EntityId] {
        &self.candidate_entity_ids
    }

    pub const fn examined_entry_count(&self) -> usize {
        self.work.examined_entry_count
    }

    pub const fn verified_entity_record_count(&self) -> usize {
        self.work.verified_entity_record_count
    }

    pub const fn verified_relation_record_count(&self) -> usize {
        self.work.verified_relation_record_count
    }

    pub const fn overflowed(&self) -> bool {
        self.overflowed
    }

    pub const fn parity_mode(&self) -> BoundedIndexParityMode {
        self.parity_mode
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BoundedRelationJoinLookupWork {
    examined_entry_count: usize,
    verified_entity_record_count: usize,
    verified_relation_record_count: usize,
}

impl BoundedRelationJoinLookupWork {
    pub(crate) const fn for_verified_candidates(candidate_count: usize) -> Self {
        Self {
            examined_entry_count: candidate_count,
            verified_entity_record_count: candidate_count + 2,
            verified_relation_record_count: candidate_count * 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundedRelationJoinLookupDenialKind {
    Execution(crate::execution::RelationalExecutionDenialCause),
    InvalidCandidateLimit,
    SnapshotUnavailable,
    IndexNotInstalled,
    WrongIndexKind,
    ExactGenerationUnavailable,
    CorruptIndexEntries,
    StorageParityMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedRelationJoinLookupDenial {
    kind: BoundedRelationJoinLookupDenialKind,
    index_id: DerivedIndexId,
}

impl BoundedRelationJoinLookupDenial {
    pub(crate) const fn new(
        kind: BoundedRelationJoinLookupDenialKind,
        index_id: DerivedIndexId,
    ) -> Self {
        Self { kind, index_id }
    }

    pub const fn kind(&self) -> BoundedRelationJoinLookupDenialKind {
        self.kind
    }

    pub const fn index_id(&self) -> DerivedIndexId {
        self.index_id
    }
}

impl std::fmt::Display for BoundedRelationJoinLookupDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "bounded relation-join lookup denied: {:?} (index {})",
            self.kind, self.index_id.0
        )
    }
}

impl std::error::Error for BoundedRelationJoinLookupDenial {}
