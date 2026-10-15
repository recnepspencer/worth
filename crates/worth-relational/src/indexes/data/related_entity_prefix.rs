//! Request-scoped exact leading-value ranges over one immutable ordered generation.
use super::{
    BoundedRelatedEntityOrderedLookupRequest, DerivedIndexGeneration, DerivedIndexGenerationId,
};
use crate::identity::data::{EntityId, KindId};
use std::sync::Arc;
use worth_foundational::facade::AspectValue;

/// Borrows exact leading ordering values; this is not a text-prefix match.
/// The existing request bounds snapshot, index, parent, child, generation and page width.
/// Its full-range `after` must be absent: only this range's opaque boundary can continue it.
pub struct BoundedRelatedEntityPrefixLookupRequest<'a> {
    pub(crate) ordered: BoundedRelatedEntityOrderedLookupRequest,
    pub(crate) prefix: &'a [AspectValue],
    pub(crate) after: Option<&'a BoundedRelatedEntityPrefixBoundary>,
}
impl<'a> BoundedRelatedEntityPrefixLookupRequest<'a> {
    pub fn new(
        ordered: BoundedRelatedEntityOrderedLookupRequest,
        prefix: &'a [AspectValue],
        after: Option<&'a BoundedRelatedEntityPrefixBoundary>,
    ) -> Self {
        Self {
            ordered,
            prefix,
            after,
        }
    }
}
/// Non-authoritative continuation retaining the original exact generation and basis.
/// It cannot be serialized, constructed from raw IDs, or reused for another range.
#[derive(Debug)]
pub struct BoundedRelatedEntityPrefixBoundary {
    pub(crate) basis: crate::visibility::snapshot_states::VisibilitySnapshotBasis,
    pub(crate) _memory: Option<worth_execution::ExecutionMemoryReservation>,
    pub(crate) runtime: u64,
    pub(crate) generation: Arc<DerivedIndexGeneration>,
    pub(crate) parent: EntityId,
    pub(crate) child_kind: KindId,
    pub(crate) prefix_len: usize,
    pub(crate) last: usize,
}
/// Complete funded page. Dropping it releases the original request's ID backing claim.
#[derive(Debug)]
pub struct BoundedRelatedEntityPrefixPage {
    pub(crate) ids: worth_execution::ExecutionArray<EntityId>,
    pub(crate) generation: DerivedIndexGenerationId,
    pub(crate) boundary: Option<BoundedRelatedEntityPrefixBoundary>,
    pub(crate) comparisons: usize,
}
impl BoundedRelatedEntityPrefixPage {
    pub fn child_entity_ids(&self) -> &[EntityId] {
        self.ids.elements()
    }
    pub fn generation_id(&self) -> DerivedIndexGenerationId {
        self.generation
    }
    pub fn next_boundary(&self) -> Option<&BoundedRelatedEntityPrefixBoundary> {
        self.boundary.as_ref()
    }
    pub fn seek_comparison_count(&self) -> usize {
        self.comparisons
    }
    /// ID-array payload only; a continuation separately retains its basis-copy reservation.
    pub fn charged_payload_bytes(&self) -> Option<u64> {
        self.ids.charged_payload_bytes()
    }
}
/// No partial page escapes a native lookup, work, control or backing refusal.
#[derive(Debug)]
pub enum BoundedRelatedEntityPrefixDenial {
    Lookup(super::BoundedRelatedEntityOrderedLookupDenial),
    InvalidPrefix,
    FieldLookup(super::BoundedEntityFieldLookupDenial),
    ForeignBoundary,
    AccountingOverflow,
    ExactBasisRequired,
    Work(worth_execution::WorkCeilingDenial),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Memory(worth_execution::MemoryLimitDenial),
    Read(crate::execution::RelationalExecutionDenialCause),
}
