//! Opaque shared RelationId observations keep their original decode admission.
use crate::identity::data::RelationId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionMemoryReservation;

pub struct InvariantRelationIds(Arc<Backing>);
struct Backing {
    values: Arc<[RelationId]>,
    memory: OnceLock<Option<ExecutionMemoryReservation>>,
}
impl InvariantRelationIds {
    pub fn as_slice(&self) -> &[RelationId] {
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
impl From<Arc<[RelationId]>> for InvariantRelationIds {
    fn from(values: Arc<[RelationId]>) -> Self {
        Self(Arc::new(Backing {
            values,
            memory: OnceLock::new(),
        }))
    }
}
impl From<Vec<RelationId>> for InvariantRelationIds {
    fn from(values: Vec<RelationId>) -> Self {
        Self::from(Arc::<[RelationId]>::from(values))
    }
}
impl<const N: usize> From<[RelationId; N]> for InvariantRelationIds {
    fn from(values: [RelationId; N]) -> Self {
        Self::from(Arc::<[RelationId]>::from(values))
    }
}
impl Clone for InvariantRelationIds {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl Default for InvariantRelationIds {
    fn default() -> Self {
        Self::from(Arc::<[RelationId]>::default())
    }
}
impl std::ops::Deref for InvariantRelationIds {
    type Target = [RelationId];
    fn deref(&self) -> &[RelationId] {
        self.as_slice()
    }
}
impl AsRef<[RelationId]> for InvariantRelationIds {
    fn as_ref(&self) -> &[RelationId] {
        self.as_slice()
    }
}
impl<'a> IntoIterator for &'a InvariantRelationIds {
    type Item = &'a RelationId;
    type IntoIter = std::slice::Iter<'a, RelationId>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
impl std::fmt::Debug for InvariantRelationIds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.values.fmt(f)
    }
}
impl PartialEq for InvariantRelationIds {
    fn eq(&self, other: &Self) -> bool {
        self.0.values == other.0.values
    }
}
impl Eq for InvariantRelationIds {}
impl Serialize for InvariantRelationIds {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.values.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for InvariantRelationIds {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Arc::<[RelationId]>::deserialize(deserializer).map(Self::from)
    }
}
