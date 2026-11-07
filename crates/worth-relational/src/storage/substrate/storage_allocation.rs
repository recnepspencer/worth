//! Identity belongs to the allocation, not to the partition that currently reaches it.
use std::sync::Arc;

// Inspection identifiers describe process-local live backing, never authority
// or durable identity. Every supported pointer fits the observation's u64.
const _: () = assert!(
    /* Proof: This is a compile-time platform check: a target whose usize pointers do not fit u64 cannot compile this allocation-identity representation. */
    usize::BITS <= u64::BITS
);

/// Inspectable copy-on-write ownership; cloning a handle preserves identity,
/// detaching its value issues a new identity. It conveys no mutation authority.
#[derive(Debug)]
pub(crate) struct StorageAllocation<T>(Arc<AllocationBacking<T>>);

#[derive(Debug)]
struct AllocationBacking<T> {
    value: T,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl<T> Clone for StorageAllocation<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> std::ops::Deref for StorageAllocation<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0.value
    }
}

impl<T> AsRef<T> for StorageAllocation<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> StorageAllocation<T> {
    pub(crate) fn new(value: T) -> Self {
        Self::from_reserved(value, None)
    }
    /// Native readmission reserves the complete immutable payload before allocation.
    pub(crate) fn from_reserved(
        value: T,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        Self(Arc::new(AllocationBacking {
            value,
            _memory: memory,
        }))
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "Addresses serve live allocation lookup and custody; canonical order uses declared identity"
    )]
    pub(crate) fn id(&self) -> u64 {
        // An observation borrows retained roots throughout its traversal. Arc
        // keeps each compared allocation alive, so addresses cannot be recycled
        // during that comparison, including when T is zero-sized (Arc header).
        Arc::as_ptr(&self.0) as usize as u64
    }
    pub(in crate::storage::substrate) fn get_mut(this: &mut Self) -> Option<&mut T> {
        Arc::get_mut(&mut this.0)
            .filter(|backing| backing._memory.is_none())
            .map(|backing| &mut backing.value)
    }
    pub(crate) fn is_unique(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }
    /// The owner payload and reference-count header, including alignment but
    /// excluding allocator-private bookkeeping and T's separately owned storage.
    pub(crate) fn layout_bytes() -> u64 {
        std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<AllocationBacking<T>>())
            .expect(/* Proof: StorageAllocation is crate-private. Its closed native monomorphizations are column/map/digest nodes and fixed owner metadata; their dynamic values are behind Vec/Arc/map headers, never input-sized inline arrays. These concrete layouts plus two usize headers are below isize::MAX on supported targets, including the preallocation callers that have not yet allocated T. */ "storage layout fits")
            .0
            .pad_to_align()
            .size() as u64
    }
}

impl<T: Clone> StorageAllocation<T> {
    pub(crate) fn make_mut(this: &mut Self) -> &mut T {
        // Funded outer payloads are immutable Native diagnostics/openings.
        // Production edits share or replace their carriers; only independently
        // admitted ordinary values can enter this unrequested mutation path.
        assert!(
            this.0._memory.is_none(),
            "funded payload needs prepared replacement"
        );
        if Arc::strong_count(&this.0) != 1 {
            *this = Self::new(this.0.value.clone());
        }
        &mut Arc::get_mut(&mut this.0)
            .expect("selected unique allocation")
            .value
    }
    pub(crate) fn unwrap_or_clone(this: Self) -> T {
        if this.0._memory.is_some() {
            // Raw wire projection must not move out the funded native backing.
            // The independently owned copy follows the receiving export contract.
            return this.0.value.clone();
        }
        match Arc::try_unwrap(this.0) {
            Ok(backing) => backing.value,
            Err(backing) => backing.value.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allocation_identity_tracks_detachment_not_equal_content() {
        let first = StorageAllocation::new(vec![1]);
        let mut second = first.clone();
        assert_eq!(first.id(), second.id());
        assert!(!second.is_unique());
        StorageAllocation::make_mut(&mut second).push(2);
        assert_ne!(first.id(), second.id());
        assert_eq!(&**first, &[1]);
        assert_eq!(&**second, &[1, 2]);
        let id = second.id();
        StorageAllocation::make_mut(&mut second).push(3);
        assert_eq!(second.id(), id);
        assert!(second.is_unique());
    }
}
