//! Intrinsic source admission follows this same private shared coordinate owner.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionMemoryReservation;

pub struct ProjectedMemberStorage(Arc<Backing>);
struct Backing {
    value: Arc<super::ProjectedMemberCoordinate>,
    memory: OnceLock<Option<ExecutionMemoryReservation>>,
}
impl ProjectedMemberStorage {
    pub(crate) fn control_bytes() -> Option<u64> {
        let layout = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<Backing>())
            .ok()?
            .0
            .pad_to_align();
        u64::try_from(layout.size()).ok()
    }
    pub(crate) fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.memory.set(memory)
    }
}
impl From<Arc<super::ProjectedMemberCoordinate>> for ProjectedMemberStorage {
    fn from(value: Arc<super::ProjectedMemberCoordinate>) -> Self {
        Self(Arc::new(Backing {
            value,
            memory: OnceLock::new(),
        }))
    }
}
impl Clone for ProjectedMemberStorage {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl std::ops::Deref for ProjectedMemberStorage {
    type Target = super::ProjectedMemberCoordinate;
    fn deref(&self) -> &super::ProjectedMemberCoordinate {
        self.as_ref()
    }
}
impl AsRef<super::ProjectedMemberCoordinate> for ProjectedMemberStorage {
    fn as_ref(&self) -> &super::ProjectedMemberCoordinate {
        &self.0.value
    }
}
impl std::fmt::Debug for ProjectedMemberStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.value.fmt(f)
    }
}
impl PartialEq for ProjectedMemberStorage {
    fn eq(&self, other: &Self) -> bool {
        self.0.value == other.0.value
    }
}
impl Eq for ProjectedMemberStorage {}
impl PartialOrd for ProjectedMemberStorage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ProjectedMemberStorage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.value.cmp(&other.0.value)
    }
}
impl std::hash::Hash for ProjectedMemberStorage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.0.value, state);
    }
}
impl Serialize for ProjectedMemberStorage {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.value.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for ProjectedMemberStorage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Arc::<super::ProjectedMemberCoordinate>::deserialize(deserializer).map(Self::from)
    }
}
