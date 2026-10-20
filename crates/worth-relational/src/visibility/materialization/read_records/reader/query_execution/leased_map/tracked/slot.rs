//! A materialized fragment and its original ticket have one final holder.
use super::QueryPacketMemory;
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::Mutex,
};

pub(super) struct FragmentSlot<T> {
    pub(super) value: Mutex<Option<T>>,
    pub(super) memory: QueryPacketMemory,
}

impl<T> FragmentSlot<T> {
    pub(super) fn new(memory: QueryPacketMemory) -> Self {
        Self {
            value: Mutex::new(None),
            memory,
        }
    }

    pub(super) fn take(&mut self) -> T {
        self.value
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .expect("every canonical query packet completed into its slot")
    }
}

impl<T> Drop for FragmentSlot<T> {
    fn drop(&mut self) {
        let value = self
            .value
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        // A refused speculative value is disposed while its original scratch
        // and result tickets still exist. Preserve the original map stop even
        // if disposal panics; a panic payload can itself panic during Drop.
        if let Err(payload) = catch_unwind(AssertUnwindSafe(|| drop(value)))
            && let Err(second) = catch_unwind(AssertUnwindSafe(|| drop(payload)))
        {
            std::mem::forget(second);
        }
    }
}

#[cfg(test)]
mod tests;
