use worth_foundational::facade::{AspectFieldLocator, AspectValue};

use crate::identity::data::{EntityId, KindId};
use crate::snapshots::data::SnapshotHandle;

use super::{DerivedIndexGenerationId, DerivedIndexId};

/// Fixed width of the bounded relation-join lane. Complete entity-field
/// selections instead use their caller's explicit finite candidate budget.
pub const MAX_BOUNDED_INDEX_CANDIDATES: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundedIndexParityMode {
    Production,
    Certification,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedEntityFieldLookupRequest {
    snapshot: SnapshotHandle,
    index_id: DerivedIndexId,
    entity_kind: KindId,
    field_locator: AspectFieldLocator,
    value: AspectValue,
    candidate_limit: usize,
}

impl BoundedEntityFieldLookupRequest {
    pub fn new(
        snapshot: SnapshotHandle,
        index_id: DerivedIndexId,
        entity_kind: KindId,
        field_locator: AspectFieldLocator,
        value: AspectValue,
        candidate_limit: usize,
    ) -> Result<Self, BoundedEntityFieldLookupDenial> {
        if candidate_limit == 0 || candidate_limit == usize::MAX {
            return Err(BoundedEntityFieldLookupDenial::new(
                BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit,
                index_id,
            ));
        }
        Ok(Self {
            snapshot,
            index_id,
            entity_kind,
            field_locator,
            value,
            candidate_limit,
        })
    }

    pub fn snapshot(&self) -> &SnapshotHandle {
        &self.snapshot
    }

    pub const fn index_id(&self) -> DerivedIndexId {
        self.index_id
    }

    pub const fn entity_kind(&self) -> KindId {
        self.entity_kind
    }

    pub fn field_locator(&self) -> &AspectFieldLocator {
        &self.field_locator
    }

    pub fn value(&self) -> &AspectValue {
        &self.value
    }

    pub const fn candidate_limit(&self) -> usize {
        self.candidate_limit
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedEntityFieldLookupOutcome {
    generation_id: DerivedIndexGenerationId,
    candidate_entity_ids: Vec<EntityId>,
    examined_entry_count: usize,
    overflowed: bool,
    parity_mode: BoundedIndexParityMode,
}

impl BoundedEntityFieldLookupOutcome {
    pub(crate) fn new(
        generation_id: DerivedIndexGenerationId,
        candidate_entity_ids: Vec<EntityId>,
        examined_entry_count: usize,
        overflowed: bool,
        parity_mode: BoundedIndexParityMode,
    ) -> Self {
        Self {
            generation_id,
            candidate_entity_ids,
            examined_entry_count,
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
        self.examined_entry_count
    }

    pub const fn overflowed(&self) -> bool {
        self.overflowed
    }

    pub const fn parity_mode(&self) -> BoundedIndexParityMode {
        self.parity_mode
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundedEntityFieldLookupDenialKind {
    InvalidCandidateLimit,
    SnapshotUnavailable,
    IndexNotInstalled,
    WrongIndexKind,
    ExactGenerationUnavailable,
    CorruptIndexEntries,
    StorageParityMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedEntityFieldLookupDenial {
    kind: BoundedEntityFieldLookupDenialKind,
    index_id: DerivedIndexId,
}

impl BoundedEntityFieldLookupDenial {
    pub(crate) const fn new(
        kind: BoundedEntityFieldLookupDenialKind,
        index_id: DerivedIndexId,
    ) -> Self {
        Self { kind, index_id }
    }

    pub const fn kind(&self) -> BoundedEntityFieldLookupDenialKind {
        self.kind
    }

    pub const fn index_id(&self) -> DerivedIndexId {
        self.index_id
    }
}

impl std::fmt::Display for BoundedEntityFieldLookupDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "bounded entity-field lookup denied: {:?} (index {})",
            self.kind, self.index_id.0
        )
    }
}

impl std::error::Error for BoundedEntityFieldLookupDenial {}
