//! Shared actual capture custody survives clones and interrupted acknowledgement.
use crate::branch::AdmittedRelationalBranchBasis;
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(in crate::durability) type OriginalCheckpointRead = (
    Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    crate::durability::medium::RelationalDurableReadProtection,
);

#[derive(Debug, Default)]
struct RetainedCapture {
    _components: Vec<AdmittedRelationalBranchBasis>,
    _original_read: Option<OriginalCheckpointRead>,
    _complete_components:
        Option<crate::durability::authority::checkpoint_components::CompleteComponentHolds>,
    region_cells: Option<
        worth_execution::ExecutionArray<Arc<crate::storage::residency::RelationalRegionResidency>>,
    >,
}
#[derive(Debug)]
struct Backing {
    held: Mutex<RetainedCapture>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone, Default)]
pub(super) struct ComponentCaptureCustody(Option<Arc<Backing>>);
impl ComponentCaptureCustody {
    /// Existing ordinary capture; grants no controlled allocation claim.
    pub(super) fn new(holds: Vec<AdmittedRelationalBranchBasis>) -> Self {
        if holds.is_empty() {
            return Self::default();
        }
        Self(Some(Arc::new(Backing {
            held: Mutex::new(RetainedCapture {
                _components: holds,
                ..Default::default()
            }),
            _memory: None,
        })))
    }

    pub(super) fn prepare_for_request(
        holds: crate::durability::authority::checkpoint_components::CompleteComponentHolds,
        region_cells: worth_execution::ExecutionArray<
            Arc<crate::storage::residency::RelationalRegionResidency>,
        >,
        original_read: Option<OriginalCheckpointRead>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Backing>())
            .ok()
            .map(|(layout, _)| layout.pad_to_align().size())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        Ok(Self(Some(Arc::new(Backing {
            held: Mutex::new(RetainedCapture {
                _components: Vec::new(),
                _complete_components: Some(holds),
                region_cells: Some(region_cells),
                _original_read: original_read,
            }),
            _memory: memory,
        }))))
    }

    pub(super) fn retains_reconstructed_regions(&self) -> Result<bool, ()> {
        let Some(owner) = &self.0 else {
            return Ok(false);
        };
        let held = match owner.held.try_lock() {
            Ok(held) => held,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(()),
        };
        Ok(held.region_cells.is_some())
    }

    pub(super) fn release(&self) {
        if let Some(owner) = &self.0 {
            let retired = std::mem::take(
                &mut *owner
                    .held
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()),
            );
            // Real ACK completion releases the actual N read and component holds.
            // Their ordinary final disposal is outside this owner's mutex.
            drop(retired);
        }
    }
}

#[cfg(test)]
mod tests;
