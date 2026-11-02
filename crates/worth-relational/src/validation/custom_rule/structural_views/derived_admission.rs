//! Successful static admission belongs only to its immutable structural view.
use crate::derived_relation::DerivedRelationHandle;
use std::sync::{Arc, Mutex};

// This is acceleration capacity, never an admission or expression-size limit.
// A full cache continues through the ordinary registered-definition gate.
pub(crate) const PROBES: u64 = 8;
pub(crate) struct DerivedAdmissionCache(Mutex<[Option<DerivedRelationHandle>; PROBES as usize]>);
impl Default for DerivedAdmissionCache {
    fn default() -> Self {
        Self(Mutex::new(std::array::from_fn(|_| None)))
    }
}
impl Clone for DerivedAdmissionCache {
    fn clone(&self) -> Self {
        Self(Mutex::new(self.0.lock().unwrap().clone()))
    }
}
impl DerivedAdmissionCache {
    pub(crate) const fn probe_work(&self) -> u64 {
        PROBES
    }
    pub(crate) fn contains(&self, handle: &DerivedRelationHandle) -> bool {
        self.0
            .lock()
            .unwrap()
            .iter()
            .flatten()
            .any(|cached| Arc::ptr_eq(&cached.entry, &handle.entry))
    }
    pub(crate) fn remember(&self, handle: &DerivedRelationHandle) {
        let mut slots = self.0.lock().unwrap();
        let mut empty = None;
        for (index, slot) in slots.iter().enumerate() {
            match slot {
                Some(cached) if Arc::ptr_eq(&cached.entry, &handle.entry) => return,
                None if empty.is_none() => empty = Some(index),
                _ => {}
            }
        }
        if let Some(index) = empty {
            slots[index] = Some(handle.clone());
        }
    }
}
#[cfg(test)]
mod tests;
