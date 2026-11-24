//! Finite pages over one exact entity-field posting generation.
use super::{DerivedIndexGeneration, DerivedIndexGenerationId};
use crate::identity::data::{EntityId, KindId};
use std::sync::Arc;

/// Opaque continuation for the same owner, basis, field, kind and comparison value.
/// Dropping it releases its optional native basis and generation pins.
#[derive(Debug)]
pub struct BoundedEntityFieldBoundary {
    pub(crate) runtime: u64,
    pub(crate) basis: crate::visibility::snapshot_states::ResidentVisibilitySnapshotBasis,
    pub(crate) generation: Arc<DerivedIndexGeneration>,
    pub(crate) kind: KindId,
    pub(crate) key:
        crate::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey,
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

pub trait BoundedEntityFieldBoundaryFieldsPort {
    fn owner_basis(&self) -> &crate::visibility::snapshot_states::ResidentVisibilitySnapshotBasis;
    fn owner_generation(&self) -> &Arc<DerivedIndexGeneration>;
    fn owner_key(
        &self,
    ) -> &crate::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey;
    fn owner_kind(&self) -> &KindId;
    fn owner_next(&self) -> &usize;
    fn owner_runtime(&self) -> &u64;
}

impl BoundedEntityFieldBoundaryFieldsPort for BoundedEntityFieldBoundary {
    fn owner_basis(&self) -> &crate::visibility::snapshot_states::ResidentVisibilitySnapshotBasis {
        &self.basis
    }
    fn owner_generation(&self) -> &Arc<DerivedIndexGeneration> {
        &self.generation
    }
    fn owner_key(
        &self,
    ) -> &crate::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey
    {
        &self.key
    }
    fn owner_kind(&self) -> &KindId {
        &self.kind
    }
    fn owner_next(&self) -> &usize {
        &self.next
    }
    fn owner_runtime(&self) -> &u64 {
        &self.runtime
    }
}

pub struct BoundedEntityFieldBoundaryParts {
    pub runtime: u64,
    pub basis: crate::visibility::snapshot_states::ResidentVisibilitySnapshotBasis,
    pub generation: Arc<DerivedIndexGeneration>,
    pub kind: KindId,
    pub key:
        crate::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey,
    pub next: usize,
    pub _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub fn assemble_bounded_entity_field_boundary(
    parts: BoundedEntityFieldBoundaryParts,
) -> BoundedEntityFieldBoundary {
    BoundedEntityFieldBoundary {
        runtime: parts.runtime,
        basis: parts.basis,
        generation: parts.generation,
        kind: parts.kind,
        key: parts.key,
        next: parts.next,
        _memory: parts._memory,
    }
}

pub fn assemble_bounded_entity_field_page(
    ids: worth_execution::ExecutionArray<EntityId>,
    generation: DerivedIndexGenerationId,
    examined: usize,
    boundary: Option<BoundedEntityFieldBoundary>,
) -> BoundedEntityFieldPage {
    BoundedEntityFieldPage {
        ids,
        generation,
        examined,
        boundary,
    }
}
