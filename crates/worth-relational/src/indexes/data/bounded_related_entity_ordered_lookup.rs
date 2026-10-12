use crate::identity::data::{EntityId, KindId};
use crate::snapshots::data::SnapshotHandle;

use super::{
    BoundedIndexParityMode, DerivedIndexGenerationId, DerivedIndexId, RelatedEntityOrderingBoundary,
};

pub const MAX_BOUNDED_RELATED_ENTITY_PAGE_WIDTH: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedRelatedEntityOrderedLookupRequest {
    snapshot: SnapshotHandle,
    index_id: DerivedIndexId,
    parent_entity_id: EntityId,
    child_kind: KindId,
    expected_generation: Option<DerivedIndexGenerationId>,
    after: Option<RelatedEntityOrderingBoundary>,
    page_width: usize,
}

impl BoundedRelatedEntityOrderedLookupRequest {
    pub fn new(
        snapshot: SnapshotHandle,
        index_id: DerivedIndexId,
        parent_entity_id: EntityId,
        child_kind: KindId,
        after: Option<RelatedEntityOrderingBoundary>,
        page_width: usize,
    ) -> Result<Self, BoundedRelatedEntityOrderedLookupDenial> {
        if page_width == 0 || page_width > MAX_BOUNDED_RELATED_ENTITY_PAGE_WIDTH {
            return Err(BoundedRelatedEntityOrderedLookupDenial::new(
                BoundedRelatedEntityOrderedLookupDenialKind::InvalidPageWidth,
                index_id,
            ));
        }
        Ok(Self {
            snapshot,
            index_id,
            parent_entity_id,
            child_kind,
            expected_generation: None,
            after,
            page_width,
        })
    }

    pub fn snapshot(&self) -> &SnapshotHandle {
        &self.snapshot
    }

    pub const fn index_id(&self) -> DerivedIndexId {
        self.index_id
    }

    pub const fn parent_entity_id(&self) -> EntityId {
        self.parent_entity_id
    }

    pub const fn child_kind(&self) -> KindId {
        self.child_kind
    }

    pub fn expect_generation(mut self, generation: DerivedIndexGenerationId) -> Self {
        self.expected_generation = Some(generation);
        self
    }

    pub const fn expected_generation(&self) -> Option<DerivedIndexGenerationId> {
        self.expected_generation
    }

    pub const fn after(&self) -> Option<&RelatedEntityOrderingBoundary> {
        self.after.as_ref()
    }

    pub const fn page_width(&self) -> usize {
        self.page_width
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedRelatedEntityOrderedLookupOutcome {
    generation_id: DerivedIndexGenerationId,
    child_entity_ids: Vec<EntityId>,
    next_boundary: Option<RelatedEntityOrderingBoundary>,
    examined_entry_count: usize,
    seek_comparison_count: usize,
    parity_mode: BoundedIndexParityMode,
}

impl BoundedRelatedEntityOrderedLookupOutcome {
    pub(crate) fn new(
        generation_id: DerivedIndexGenerationId,
        child_entity_ids: Vec<EntityId>,
        next_boundary: Option<RelatedEntityOrderingBoundary>,
        examined_entry_count: usize,
        seek_comparison_count: usize,
        parity_mode: BoundedIndexParityMode,
    ) -> Self {
        Self {
            generation_id,
            child_entity_ids,
            next_boundary,
            examined_entry_count,
            seek_comparison_count,
            parity_mode,
        }
    }

    pub const fn generation_id(&self) -> DerivedIndexGenerationId {
        self.generation_id
    }

    pub fn child_entity_ids(&self) -> &[EntityId] {
        &self.child_entity_ids
    }

    pub const fn next_boundary(&self) -> Option<&RelatedEntityOrderingBoundary> {
        self.next_boundary.as_ref()
    }

    pub fn into_next_boundary(self) -> Option<RelatedEntityOrderingBoundary> {
        self.next_boundary
    }

    pub const fn examined_entry_count(&self) -> usize {
        self.examined_entry_count
    }

    pub const fn seek_comparison_count(&self) -> usize {
        self.seek_comparison_count
    }

    pub const fn has_more(&self) -> bool {
        self.next_boundary.is_some()
    }

    pub const fn parity_mode(&self) -> BoundedIndexParityMode {
        self.parity_mode
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundedRelatedEntityOrderedLookupDenialKind {
    Execution(crate::execution::RelationalExecutionDenialCause),
    InvalidPageWidth,
    SnapshotUnavailable,
    IndexNotInstalled,
    WrongIndexKind,
    ExactGenerationUnavailable,
    ExpectedGenerationMismatch,
    ForeignBoundary,
    CorruptIndexEntries,
    StorageParityMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedRelatedEntityOrderedLookupDenial {
    kind: BoundedRelatedEntityOrderedLookupDenialKind,
    index_id: DerivedIndexId,
}

impl BoundedRelatedEntityOrderedLookupDenial {
    pub(crate) const fn new(
        kind: BoundedRelatedEntityOrderedLookupDenialKind,
        index_id: DerivedIndexId,
    ) -> Self {
        Self { kind, index_id }
    }

    pub const fn kind(&self) -> BoundedRelatedEntityOrderedLookupDenialKind {
        self.kind
    }

    pub const fn index_id(&self) -> DerivedIndexId {
        self.index_id
    }
}

impl std::fmt::Display for BoundedRelatedEntityOrderedLookupDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "bounded related-entity ordered lookup denied: {:?} (index {})",
            self.kind, self.index_id.0
        )
    }
}

impl std::error::Error for BoundedRelatedEntityOrderedLookupDenial {}
