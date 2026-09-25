//! Identity belongs to the allocation, not to the partition that currently reaches it.
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

static NEXT_ALLOCATION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
struct Allocation<T> {
    id: u64,
    value: T,
}

impl<T: Clone> Clone for Allocation<T> {
    fn clone(&self) -> Self {
        Self::new(self.value.clone())
    }
}

impl<T> Allocation<T> {
    fn new(value: T) -> Self {
        let id = NEXT_ALLOCATION_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("native storage allocation identity exhausted");
        Self { id, value }
    }
}

/// Inspectable copy-on-write ownership; cloning a handle preserves identity,
/// detaching its value issues a new identity. It conveys no mutation authority.
#[derive(Debug)]
pub(crate) struct StorageAllocation<T>(Arc<Allocation<T>>);

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
        Self(Arc::new(Allocation::new(value)))
    }
    pub(crate) fn id(&self) -> u64 {
        self.0.id
    }
    pub(crate) fn is_unique(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }
    /// The owner payload and reference-count header, including alignment but
    /// excluding allocator-private bookkeeping and T's separately owned storage.
    pub(crate) fn layout_bytes() -> u64 {
        std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Allocation<T>>())
            .expect("storage layout fits")
            .0
            .pad_to_align()
            .size() as u64
    }
}

impl<T: Clone> StorageAllocation<T> {
    pub(crate) fn make_mut(this: &mut Self) -> &mut T {
        &mut Arc::make_mut(&mut this.0).value
    }
    pub(crate) fn unwrap_or_clone(this: Self) -> T {
        Arc::unwrap_or_clone(this.0).value
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
