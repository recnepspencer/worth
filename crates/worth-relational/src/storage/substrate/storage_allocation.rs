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
pub(crate) struct StorageAllocation<T>(Arc<T>);

impl<T> Clone for StorageAllocation<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> std::ops::Deref for StorageAllocation<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> AsRef<T> for StorageAllocation<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> StorageAllocation<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(Arc::new(value))
    }
    pub(crate) fn id(&self) -> u64 {
        // An observation borrows retained roots throughout its traversal. Arc
        // keeps each compared allocation alive, so addresses cannot be recycled
        // during that comparison, including when T is zero-sized (Arc header).
        Arc::as_ptr(&self.0) as usize as u64
    }
    pub(crate) fn is_unique(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }
    /// The owner payload and reference-count header, including alignment but
    /// excluding allocator-private bookkeeping and T's separately owned storage.
    pub(crate) fn layout_bytes() -> u64 {
        std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<T>())
            .expect(/* Proof: StorageAllocation is crate-private. Its closed native monomorphizations are column/map/digest nodes and fixed owner metadata; their dynamic values are behind Vec/Arc/map headers, never input-sized inline arrays. These concrete layouts plus two usize headers are below isize::MAX on supported targets, including the preallocation callers that have not yet allocated T. */ "storage layout fits")
            .0
            .pad_to_align()
            .size() as u64
    }
}

impl<T: Clone> StorageAllocation<T> {
    pub(crate) fn make_mut(this: &mut Self) -> &mut T {
        Arc::make_mut(&mut this.0)
    }
    pub(crate) fn unwrap_or_clone(this: Self) -> T {
        Arc::unwrap_or_clone(this.0)
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
