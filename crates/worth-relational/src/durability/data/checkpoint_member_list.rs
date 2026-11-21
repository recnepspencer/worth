//! The mixed captured member roster shares actual fixed storage and its own bill.
use super::{RelationalCheckpointMember, RelationalNativeCheckpointCaptureDenial as Denial};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionArray, ExecutionMemoryReservation};
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
    pub(crate) fn capture<'request, 'authority: 'request>(
        meter: &mut impl crate::durability::capture_meter::CaptureMeter<'request, 'authority>,
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
        let memory = meter.reserve(bytes as u64).map_err(Denial::Request)?;
        let mut building = meter
            .array(count)
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
