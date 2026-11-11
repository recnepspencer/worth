//! One protected Home image supplies eligibility and later actual World admission.
use super::{
    RuntimeWorldDurableMedium, WorldCheckpointImage, WorldDurableMediumBinding,
    WorldReadmissionDenial as Denial,
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
mod data;
pub(in crate::durability) mod native_catalog;
pub use data::PreparedWorldHomeData;

/// Bound only after Home descriptor publication; no capability retains this owner.
#[derive(Debug)]
pub struct PreparedWorldHomeSource {
    binding: Arc<WorldDurableMediumBinding>,
    data: PreparedWorldHomeData,
    _memory: Option<ExecutionMemoryReservation>,
}
impl PreparedWorldHomeSource {
    pub(crate) fn binding(&self) -> &Arc<WorldDurableMediumBinding> {
        &self.binding
    }
    pub(crate) fn recovered(&self) -> &WorldCheckpointImage {
        self.data.recovered()
    }
}
impl PreparedWorldHomeData {
    /// Attaches the SAME admitted source to its actual Home writer, without rereading.
    pub fn bind(
        self,
        medium: Arc<dyn RuntimeWorldDurableMedium>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedWorldHomeSource, Denial> {
        let actual = medium.binding();
        if actual.owner() != self.expected().owner() {
            return Err(Denial::ForeignOwner);
        }
        if actual.incarnation() != self.expected().incarnation() {
            return Err(Denial::JournalSourceMismatch);
        }
        if actual.program_revision() != self.expected().program_revision() {
            return Err(Denial::ProgramRevisionMismatch);
        }
        let memory = request
            .reserve_retained_memory(size_of::<PreparedWorldHomeSource>() as u64)
            .map_err(|cause| Denial::Medium(super::WorldDurableMediumDenial::Request(cause)))?;
        let binding = WorldDurableMediumBinding::new(medium, request).map_err(Denial::Medium)?;
        binding
            .admit_recovered_checkpoint_coverage(self.coverage().clone())
            .map_err(Denial::Medium)?;
        Ok(PreparedWorldHomeSource {
            binding,
            data: self,
            _memory: memory,
        })
    }
}
