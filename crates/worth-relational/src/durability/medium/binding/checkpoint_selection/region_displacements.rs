//! Retain displaced inert region metadata outside every residency lock.
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
use crate::storage::residency::{RecordedRegionCheckpoint, RetiredRegionCheckpointBindings};
#[derive(Debug)]
pub(super) struct RegionDisplacement {
    pub(super) ordinary: Option<RecordedRegionCheckpoint>,
    pub(super) controlled: Option<RetiredRegionCheckpointBindings>,
}
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
#[derive(Debug)]
pub(super) struct RegionDisplacements {
    values: Vec<RegionDisplacement>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl RegionDisplacements {
    pub(super) fn prepare(
        request: &ExecutionRequest<'_, '_>,
        count: usize,
    ) -> Result<Self, Denial> {
        let bytes = std::alloc::Layout::array::<RegionDisplacement>(count)
            .map_err(|_| {
                Denial::Request(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)
            })?
            .size();
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let mut values = Vec::with_capacity(count);
        values.resize_with(count, || RegionDisplacement {
            ordinary: None,
            controlled: None,
        });
        Ok(Self {
            values,
            _memory: memory,
        })
    }
    pub(super) fn slot(&mut self, index: usize) -> &mut RegionDisplacement {
        self.values
            .get_mut(index)
            .expect("original capture pre-admitted every region placement slot")
    }
}
