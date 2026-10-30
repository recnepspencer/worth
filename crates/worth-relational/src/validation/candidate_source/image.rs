//! Native owns published Source associations; provider payloads remain opaque.
use super::{ManagedIndexSourceAvailability as Availability, ManagedIndexSourceProgram as Program};
use crate::identity::data::EntityId;
use std::{any::Any, sync::Arc};

#[derive(Clone, Default)]
pub struct ManagedIndexBranchImage {
    pub(crate) roots: im::HashMap<(Program, EntityId), Arc<ManagedIndexPublishedSourceRoot>>,
}
pub struct ManagedIndexPublishedSourceRoot {
    pub(crate) program: Program,
    pub(crate) scope: EntityId,
    pub(crate) payload: Arc<dyn Any + Send + Sync>,
    pub(crate) availability: Availability,
    pub(crate) revision: crate::history::data::CommitId,
    pub(crate) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ManagedIndexBranchImage {
    /// Material retention only; this does not mint an invariant or public loan.
    pub fn retained(
        &self,
        program: &Program,
        scope: EntityId,
    ) -> Option<&Arc<ManagedIndexPublishedSourceRoot>> {
        self.roots.get(&(program.clone(), scope))
    }
}
impl ManagedIndexPublishedSourceRoot {
    pub fn payload(&self) -> &Arc<dyn Any + Send + Sync> {
        &self.payload
    }
    pub const fn physical_availability(&self) -> Availability {
        self.availability
    }
}
