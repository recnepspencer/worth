//! Read-only physical allocation traversal for persistent aspect state.
use std::alloc::Layout;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static NEXT_ALLOCATION_BIRTH_SEQ: AtomicU64 = AtomicU64::new(1);

/// Non-wrapping monotonic birth sequence for foundational allocations.
/// Returns `None` if exhausted so callers fall back to exact refcount checks.
pub(crate) fn next_allocation_birth_seq() -> Option<u64> {
    let mut current = NEXT_ALLOCATION_BIRTH_SEQ.load(Ordering::Relaxed);
    loop {
        let next = current.checked_add(1)?;
        match NEXT_ALLOCATION_BIRTH_SEQ.compare_exchange_weak(
            current,
            next,
            Ordering::AcqRel,
            Ordering::Relaxed,
        ) {
            Ok(seq) => return Some(seq),
            Err(observed) => current = observed,
        }
    }
}

/// Exclusive upper bound on the `birth_seq` of every foundational allocation
/// already constructed before this call. Any allocation with `birth_seq >=
/// watermark` was constructed strictly after this watermark was captured.
pub fn current_foundational_allocation_watermark() -> Option<u64> {
    let next = NEXT_ALLOCATION_BIRTH_SEQ.load(Ordering::Acquire);
    (next < u64::MAX).then_some(next)
}

// Inspection identifiers describe process-local live backing, never authority
// or durable identity. Every supported pointer fits the observation's u64.
const _: () = assert!(
    /* Proof: This is a compile-time platform check: a target whose usize pointers do not fit u64 cannot compile this allocation-identity representation. */
    usize::BITS <= u64::BITS
);

/// Classification of a live heap allocation reached from persistent aspect state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FoundationalAllocationKind {
    /// Persistent map node, admitted value cell, struct field node, or value backing.
    ValueBacking,
    /// Immutable shared aspect contract definition.
    SchemaContract,
}

/// One live heap allocation observed during a read-only traversal.
///
/// `id` is the allocation's process-local live address, valid only while the
/// borrowed owners remain alive during the traversal. It grants no mutation,
/// reconstruction, or durable authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoundationalAllocationObservation {
    pub id: u64,
    pub bytes: u64,
    pub unique: bool,
    pub strong_count: usize,
    pub birth_seq: Option<u64>,
    pub kind: FoundationalAllocationKind,
}

/// Read-only visitor for physical heap allocations.
pub trait FoundationalAllocationVisitor {
    /// Return `true` to descend into allocations owned by this allocation.
    fn visit(&mut self, allocation: FoundationalAllocationObservation) -> bool;
}

pub(crate) fn arc_allocation_bytes<T>() -> Option<usize> {
    let (layout, _) = Layout::new::<[usize; 2]>()
        .extend(Layout::new::<T>())
        .ok()?;
    Some(layout.pad_to_align().size())
}

pub(crate) fn arc_id<T>(arc: &Arc<T>) -> u64 {
    Arc::as_ptr(arc) as usize as u64
}

pub(crate) fn ptr_id<T: ?Sized>(ptr: *const T) -> u64 {
    ptr as *const () as usize as u64
}

pub(crate) fn visit_string_buffer(
    text: &String,
    unique: bool,
    owner_birth_seq: Option<u64>,
    visitor: &mut dyn FoundationalAllocationVisitor,
) {
    if text.capacity() == 0 {
        return;
    }
    visitor.visit(FoundationalAllocationObservation {
        id: ptr_id(text.as_ptr()),
        bytes: text.capacity() as u64,
        unique,
        strong_count: 1,
        birth_seq: owner_birth_seq,
        kind: FoundationalAllocationKind::ValueBacking,
    });
}

pub(crate) fn visit_bytes_buffer(
    bytes: &[u8],
    capacity: usize,
    unique: bool,
    owner_birth_seq: Option<u64>,
    visitor: &mut dyn FoundationalAllocationVisitor,
) {
    if capacity == 0 {
        return;
    }
    visitor.visit(FoundationalAllocationObservation {
        id: ptr_id(bytes.as_ptr()),
        bytes: capacity as u64,
        unique,
        strong_count: 1,
        birth_seq: owner_birth_seq,
        kind: FoundationalAllocationKind::ValueBacking,
    });
}
