//! One receiver reservation stages all funded rows before one atomic install.
use super::*;
use crate::lifecycle::{interim_checkpoint::charge, WorldInterimCheckpointRefusal as Stop};
use worth_execution::{ExecutionArray, ExecutionRequest};

pub(crate) struct MemoryRegistryReadmission {
    registry: Arc<Registry>,
    completed: bool,
}
impl CompositeHistoryCatalog {
    pub(crate) fn reserve_memory_readmission(&self) -> Result<MemoryRegistryReadmission, Stop> {
        let mut records = self
            .originals
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match records.receiver {
            ReceiverAdmission::Available if records.slots.is_empty() => {}
            ReceiverAdmission::Available => return Err(Stop::BindingMismatch),
            ReceiverAdmission::Preparing
            | ReceiverAdmission::Installed
            | ReceiverAdmission::Refused => return Err(Stop::ReceiverAlreadyBound),
        }
        records.receiver = ReceiverAdmission::Preparing;
        Ok(MemoryRegistryReadmission {
            registry: self.originals.0.clone(),
            completed: false,
        })
    }
}
impl MemoryRegistryReadmission {
    pub(crate) fn install(
        mut self,
        rows: ExecutionArray<Arc<MemoryPublicationRecord>>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Stop> {
        let bytes = std::alloc::Layout::array::<Slot>(rows.len())
            .map_err(|_| Stop::SizeOverflow)?
            .size();
        let funding = request
            .reserve_retained_memory(bytes as u64)
            .map_err(Stop::Request)?;
        charge(request, rows.len())?;
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(rows.len())
            .map_err(|_| Stop::AllocationFailed)?;
        if slots.capacity() != rows.len() {
            return Err(Stop::SizeOverflow);
        }
        slots.extend(rows.iter().cloned().map(Slot::Original));
        let mut records = self
            .registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(records.receiver, ReceiverAdmission::Preparing) || !records.slots.is_empty() {
            return Err(Stop::BindingMismatch);
        }
        records.slots = slots;
        records._capacity_funding = funding;
        records.receiver = ReceiverAdmission::Installed;
        self.completed = true;
        Ok(())
    }
}
impl Drop for MemoryRegistryReadmission {
    fn drop(&mut self) {
        if !self.completed {
            let mut records = self
                .registry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if matches!(records.receiver, ReceiverAdmission::Preparing) {
                records.receiver = ReceiverAdmission::Refused;
            }
        }
    }
}
