//! Opaque shared EntityId observations keep their original decode admission.
use crate::admitted_shared::AdmittedShared;
use crate::identity::data::EntityId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InvariantEntityIds(AdmittedShared<[EntityId]>);

impl std::fmt::Debug for InvariantEntityIds {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}
impl InvariantEntityIds {
    pub fn as_slice(&self) -> &[EntityId] {
        &self.0
    }
}
impl From<Arc<[EntityId]>> for InvariantEntityIds {
    fn from(values: Arc<[EntityId]>) -> Self {
        Self(AdmittedShared::new(values))
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

/// Operations used by the Relational owners.
pub trait InvariantEntityIdsOwnerPort: Sized {
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>>;
    fn control_bytes() -> Option<u64>;
}

impl InvariantEntityIdsOwnerPort for InvariantEntityIds {
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.retain_decoded_memory(memory)
    }

    fn control_bytes() -> Option<u64> {
        AdmittedShared::<[EntityId]>::control_bytes()
    }
}
