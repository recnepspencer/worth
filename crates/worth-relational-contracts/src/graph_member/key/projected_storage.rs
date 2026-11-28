//! Intrinsic source admission follows this same private shared coordinate owner.
use crate::admitted_shared::AdmittedShared;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectedMemberStorage(AdmittedShared<super::ProjectedMemberCoordinate>);

impl std::fmt::Debug for ProjectedMemberStorage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}

impl From<Arc<super::ProjectedMemberCoordinate>> for ProjectedMemberStorage {
    fn from(value: Arc<super::ProjectedMemberCoordinate>) -> Self {
        Self(AdmittedShared::new(value))
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
        &self.0
    }
}

/// Operations used by the Relational owners.
pub trait ProjectedMemberStorageOwnerPort: Sized {
    fn control_bytes() -> Option<u64>;
    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>>;
}

impl ProjectedMemberStorageOwnerPort for ProjectedMemberStorage {
    fn control_bytes() -> Option<u64> {
        AdmittedShared::<super::ProjectedMemberCoordinate>::control_bytes()
    }

    fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.retain_decoded_memory(memory)
    }
}
