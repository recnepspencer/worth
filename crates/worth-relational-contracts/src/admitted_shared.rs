//! Shared immutable payloads retain their original decode admission.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    alloc::Layout,
    fmt,
    ops::Deref,
    sync::{Arc, OnceLock},
};
use worth_execution::ExecutionMemoryReservation;

pub(crate) struct AdmittedShared<T: ?Sized>(Arc<Backing<T>>);

struct Backing<T: ?Sized> {
    value: Arc<T>,
    // Release payload custody before releasing its reservation.
    memory: OnceLock<Option<ExecutionMemoryReservation>>,
}

impl<T: ?Sized> AdmittedShared<T> {
    pub(crate) fn new(value: Arc<T>) -> Self {
        Self(Arc::new(Backing {
            value,
            memory: OnceLock::new(),
        }))
    }

    pub(crate) fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.memory.set(memory)
    }

    pub(crate) fn control_bytes() -> Option<u64> {
        let layout = Layout::new::<[usize; 2]>()
            .extend(Layout::new::<Backing<T>>())
            .ok()?
            .0
            .pad_to_align();
        u64::try_from(layout.size()).ok()
    }
}

impl<T: ?Sized> Clone for AdmittedShared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> Deref for AdmittedShared<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0.value
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for AdmittedShared<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, formatter)
    }
}

impl<T: ?Sized + PartialEq> PartialEq for AdmittedShared<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: ?Sized + Eq> Eq for AdmittedShared<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for AdmittedShared<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(other)
    }
}

impl<T: ?Sized + Ord> Ord for AdmittedShared<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (**self).cmp(other)
    }
}

impl<T: ?Sized + std::hash::Hash> std::hash::Hash for AdmittedShared<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: ?Sized + Serialize> Serialize for AdmittedShared<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (**self).serialize(serializer)
    }
}

impl<'de, T: ?Sized> Deserialize<'de> for AdmittedShared<T>
where
    Arc<T>: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Arc::deserialize(deserializer).map(Self::new)
    }
}
