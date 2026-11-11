//! One shared custody cell per original load-head history registration.
use crate::durability::medium::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial, NativeSourceMemberState,
    RelationalDurableColdSource, RelationalDurableMemberCustody,
    RelationalDurableMemberRetirementObservation as Observation,
};
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};

#[derive(Debug)]
pub(crate) struct RecordHistoryCustody {
    physical: Mutex<NativeSourceMemberState>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl RecordHistoryCustody {
    pub(crate) fn reserve(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<ExecutionMemoryReservation>, WorkCeilingDenial> {
        let bytes = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("fixed custody layout fits")
            .0
            .pad_to_align()
            .size();
        request.reserve_retained_memory(bytes as u64)
    }
    pub(crate) fn new(
        source: Arc<dyn RelationalDurableColdSource>,
        token: RelationalDurableMemberCustody,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Arc<Self> {
        Arc::new(Self {
            physical: Mutex::new(NativeSourceMemberState::new(source, token)),
            _memory: memory,
        })
    }
    pub(crate) fn source(&self) -> Option<Arc<dyn RelationalDurableColdSource>> {
        self.physical
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .source()
            .cloned()
    }
    pub(crate) fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Denial> {
        let token = self
            .physical
            .try_lock()
            .map_err(|_| Denial::Busy)?
            .preflight_snapshot(source)?;
        if let Some(token) = token {
            token
                .preflight_retirement(source)
                .map_err(Denial::Refused)?;
        }
        Ok(())
    }
    pub(crate) fn loan_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        self.physical
            .try_lock()
            .map_err(|_| Denial::Busy)?
            .loan(handoff)
    }
    pub(crate) fn return_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        self.physical
            .try_lock()
            .map_err(|_| Denial::Busy)?
            .return_handoff(handoff, source)
    }
}
