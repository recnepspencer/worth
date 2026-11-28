//! Opaque shared RelationId observations keep their original decode admission.
use crate::admitted_shared::AdmittedShared;
use crate::identity::data::RelationId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InvariantRelationIds(AdmittedShared<[RelationId]>);

impl std::fmt::Debug for InvariantRelationIds {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}
impl InvariantRelationIds {
    pub fn as_slice(&self) -> &[RelationId] {
        &self.0
    }
}
impl From<Arc<[RelationId]>> for InvariantRelationIds {
    fn from(values: Arc<[RelationId]>) -> Self {
        Self(AdmittedShared::new(values))
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

/// Operations used by the Relational owners.
pub trait InvariantRelationIdsOwnerPort: Sized {
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>>;
    fn control_bytes() -> Option<u64>;
}

impl InvariantRelationIdsOwnerPort for InvariantRelationIds {
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.retain_decoded_memory(memory)
    }

    fn control_bytes() -> Option<u64> {
        AdmittedShared::<[RelationId]>::control_bytes()
    }
}
