//! Opaque shared EntityId observations keep their original decode admission.
use crate::identity::data::EntityId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionMemoryReservation;

pub struct InvariantEntityIds(Arc<Backing>);
struct Backing {
    values: Arc<[EntityId]>,
    memory: OnceLock<Option<ExecutionMemoryReservation>>,
}
impl InvariantEntityIds {
    pub fn as_slice(&self) -> &[EntityId] {
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
impl From<Arc<[EntityId]>> for InvariantEntityIds {
    fn from(values: Arc<[EntityId]>) -> Self {
        Self(Arc::new(Backing {
            values,
            memory: OnceLock::new(),
        }))
    }
}
impl From<Vec<EntityId>> for InvariantEntityIds {
    fn from(values: Vec<EntityId>) -> Self {
        Self::from(Arc::<[EntityId]>::from(values))
    }
}
impl<const N: usize> From<[EntityId; N]> for InvariantEntityIds {
    fn from(values: [EntityId; N]) -> Self {
        Self::from(Arc::<[EntityId]>::from(values))
    }
}
impl Clone for InvariantEntityIds {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl Default for InvariantEntityIds {
    fn default() -> Self {
        Self::from(Arc::<[EntityId]>::default())
    }
}
impl std::ops::Deref for InvariantEntityIds {
    type Target = [EntityId];
    fn deref(&self) -> &[EntityId] {
        self.as_slice()
    }
}
impl AsRef<[EntityId]> for InvariantEntityIds {
    fn as_ref(&self) -> &[EntityId] {
        self.as_slice()
    }
}
impl<'a> IntoIterator for &'a InvariantEntityIds {
    type Item = &'a EntityId;
    type IntoIter = std::slice::Iter<'a, EntityId>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
impl std::fmt::Debug for InvariantEntityIds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.values.fmt(f)
    }
}
impl PartialEq for InvariantEntityIds {
    fn eq(&self, other: &Self) -> bool {
        self.0.values == other.0.values
    }
}
impl Eq for InvariantEntityIds {}
impl Serialize for InvariantEntityIds {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.values.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for InvariantEntityIds {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Arc::<[EntityId]>::deserialize(deserializer).map(Self::from)
    }
}
