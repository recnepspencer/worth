//! The mixed captured member roster shares actual fixed storage and its own bill.
use super::{RelationalCheckpointMember, RelationalNativeCheckpointCaptureDenial as Denial};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
};
#[derive(Debug)]
struct Backing {
    entries: ExecutionArray<RelationalCheckpointMember>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct CapturedCheckpointMembers(Option<Arc<Backing>>);
impl CapturedCheckpointMembers {
    pub(crate) fn entries(&self) -> &[RelationalCheckpointMember] {
        self.0
            .as_ref()
            .map_or(&[], |owner| owner.entries.elements())
    }
    pub(crate) fn capture(
        request: &ExecutionRequest<'_, '_>,
        count: usize,
        entries: impl IntoIterator<Item = RelationalCheckpointMember>,
    ) -> Result<Self, Denial> {
        let bytes = Layout::new::<(usize, usize)>()
            .extend(Layout::new::<Backing>())
            .map_err(|_| {
                Denial::Request(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)
            })?
            .0
            .pad_to_align()
            .size();
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let mut building = ExecutionRequestArrayBuilder::allocate(*request, count)
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        for entry in entries {
            building
                .push(entry)
                .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        }
        let entries = building
            .seal()
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        Ok(Self(Some(Arc::new(Backing {
            entries,
            _memory: memory,
        }))))
    }
}
