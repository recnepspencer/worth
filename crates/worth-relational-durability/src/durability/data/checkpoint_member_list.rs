//! The mixed captured member roster shares actual fixed storage and its own bill.
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use std::{alloc::Layout, sync::Arc};
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::data::checkpoint_member::RelationalCheckpointMember;
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
        Self::capture_fallible(request, count, entries.into_iter().map(Ok))
    }

    pub(crate) fn capture_fallible(
        request: &ExecutionRequest<'_, '_>,
        count: usize,
        entries: impl IntoIterator<Item = Result<RelationalCheckpointMember, Denial>>,
    ) -> Result<Self, Denial> {
        let bytes = Layout::new::<(usize, usize)>()
            .extend(Layout::new::<Backing>())
            .map_err(|_| {
                Denial::Request(contracts::execution::denial_cause::RelationalExecutionDenialCause::MemoryOverflow)
            })?
            .0
            .pad_to_align()
            .size();
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let mut building = ExecutionRequestArrayBuilder::allocate(*request, count).map_err(
            contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop,
        )?;
        for entry in entries {
            building.push(entry?).map_err(
                contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop,
            )?;
        }
        let entries = building.seal().map_err(
            contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop,
        )?;
        Ok(Self(Some(Arc::new(Backing {
            entries,
            _memory: memory,
        }))))
    }
}
