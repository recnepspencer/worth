//! Opaque shared PartitionId observations keep their original decode admission.
use crate::admitted_shared::AdmittedShared;
use crate::identity::data::PartitionId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InvariantPartitionIds(AdmittedShared<[PartitionId]>);

impl std::fmt::Debug for InvariantPartitionIds {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}
impl InvariantPartitionIds {
    pub fn as_slice(&self) -> &[PartitionId] {
        &self.0
    }
}
impl From<Arc<[PartitionId]>> for InvariantPartitionIds {
    fn from(values: Arc<[PartitionId]>) -> Self {
        Self(AdmittedShared::new(values))
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

/// Operations used by the Relational owners.
pub trait InvariantPartitionIdsOwnerPort: Sized {
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>>;
    fn control_bytes() -> Option<u64>;
}

impl InvariantPartitionIdsOwnerPort for InvariantPartitionIds {
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.retain_decoded_memory(memory)
    }

    fn control_bytes() -> Option<u64> {
        AdmittedShared::<[PartitionId]>::control_bytes()
    }
}
