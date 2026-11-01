//! Retain displaced inert region metadata outside every residency lock.
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
use crate::storage::residency::RecordedRegionCheckpoint;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
#[derive(Debug)]
pub(super) struct RegionDisplacements {
    values: Vec<Option<RecordedRegionCheckpoint>>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl RegionDisplacements {
    pub(super) fn prepare(
        request: &ExecutionRequest<'_, '_>,
        count: usize,
    ) -> Result<Self, Denial> {
        let bytes = std::alloc::Layout::array::<Option<RecordedRegionCheckpoint>>(count)
            .map_err(|_| {
                Denial::Request(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)
            })?
            .size();
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let mut values = Vec::with_capacity(count);
        values.resize_with(count, || None);
        Ok(Self {
            values,
            _memory: memory,
        })
    }
    pub(super) fn slot(&mut self, index: usize) -> &mut Option<RecordedRegionCheckpoint> {
        self.values
            .get_mut(index)
            .expect("original capture pre-admitted every region placement slot")
    }
}
