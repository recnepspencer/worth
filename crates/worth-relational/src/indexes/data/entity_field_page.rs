//! Finite pages over one exact entity-field posting generation.
use super::{DerivedIndexGeneration, DerivedIndexGenerationId};
use crate::identity::data::{EntityId, KindId};
use std::sync::Arc;

/// Opaque continuation for the same owner, basis, field, kind and comparison value.
/// Dropping it releases its optional native basis and generation pins.
#[derive(Debug)]
pub struct BoundedEntityFieldBoundary {
    pub(crate) runtime: u64,
    pub(crate) basis: crate::visibility::snapshot_states::VisibilitySnapshotBasis,
    pub(crate) generation: Arc<DerivedIndexGeneration>,
    pub(crate) kind: KindId,
    pub(crate) key: crate::storage::data::AuthoritativeFieldComparisonKey,
    pub(crate) next: usize,
    pub(crate) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// A complete funded page; its width bounds examined postings, including other kinds.
#[derive(Debug)]
pub struct BoundedEntityFieldPage {
    pub(crate) ids: worth_execution::ExecutionArray<EntityId>,
    pub(crate) generation: DerivedIndexGenerationId,
    pub(crate) examined: usize,
    pub(crate) boundary: Option<BoundedEntityFieldBoundary>,
}
impl BoundedEntityFieldPage {
    pub fn candidate_entity_ids(&self) -> &[EntityId] {
        self.ids.elements()
    }
    pub fn generation_id(&self) -> DerivedIndexGenerationId {
        self.generation
    }
    pub fn examined_entry_count(&self) -> usize {
        self.examined
    }
    pub fn next_boundary(&self) -> Option<&BoundedEntityFieldBoundary> {
        self.boundary.as_ref()
    }
    pub fn into_next_boundary(self) -> Option<BoundedEntityFieldBoundary> {
        self.boundary
    }
    pub fn charged_payload_bytes(&self) -> Option<u64> {
        self.ids.charged_payload_bytes()
    }
}

/// A refused page publishes neither partial IDs nor an advanced continuation.
#[derive(Debug)]
pub enum BoundedEntityFieldPageStop<E> {
    Lookup(super::BoundedEntityFieldLookupDenial),
    Admission(E),
    ForeignBoundary,
    AccountingOverflow,
    Work(worth_execution::WorkCeilingDenial),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Memory(worth_execution::MemoryLimitDenial),
}
