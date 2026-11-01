//! Opaque shared PartitionId observations keep their original decode admission.
use crate::identity::data::PartitionId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionMemoryReservation;

pub struct InvariantPartitionIds(Arc<Backing>);
struct Backing {
    values: Arc<[PartitionId]>,
    memory: OnceLock<Option<ExecutionMemoryReservation>>,
}
impl InvariantPartitionIds {
    pub fn as_slice(&self) -> &[PartitionId] {
        &self.0.values
    }
    pub(crate) fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.memory.set(memory)
    }
    pub(crate) fn control_bytes() -> Option<u64> {
        let layout = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<Backing>())
            .ok()?
            .0
            .pad_to_align();
        u64::try_from(layout.size()).ok()
    }
}
impl From<Arc<[PartitionId]>> for InvariantPartitionIds {
    fn from(values: Arc<[PartitionId]>) -> Self {
        Self(Arc::new(Backing {
            values,
            memory: OnceLock::new(),
        }))
    }
}
impl From<Vec<PartitionId>> for InvariantPartitionIds {
    fn from(values: Vec<PartitionId>) -> Self {
        Self::from(Arc::<[PartitionId]>::from(values))
    }
}
impl<const N: usize> From<[PartitionId; N]> for InvariantPartitionIds {
    fn from(values: [PartitionId; N]) -> Self {
        Self::from(Arc::<[PartitionId]>::from(values))
    }
}
impl Clone for InvariantPartitionIds {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl Default for InvariantPartitionIds {
    fn default() -> Self {
        Self::from(Arc::<[PartitionId]>::default())
    }
}
impl std::ops::Deref for InvariantPartitionIds {
    type Target = [PartitionId];
    fn deref(&self) -> &[PartitionId] {
        self.as_slice()
    }
}
impl AsRef<[PartitionId]> for InvariantPartitionIds {
    fn as_ref(&self) -> &[PartitionId] {
        self.as_slice()
    }
}
impl<'a> IntoIterator for &'a InvariantPartitionIds {
    type Item = &'a PartitionId;
    type IntoIter = std::slice::Iter<'a, PartitionId>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
impl std::fmt::Debug for InvariantPartitionIds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.values.fmt(f)
    }
}
impl PartialEq for InvariantPartitionIds {
    fn eq(&self, other: &Self) -> bool {
        self.0.values == other.0.values
    }
}
impl Eq for InvariantPartitionIds {}
impl Serialize for InvariantPartitionIds {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.values.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for InvariantPartitionIds {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Arc::<[PartitionId]>::deserialize(deserializer).map(Self::from)
    }
}
