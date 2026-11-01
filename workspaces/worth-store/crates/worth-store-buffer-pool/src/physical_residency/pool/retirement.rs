//! Discharge actual original leases without waiting or draining the pool.
use super::*;
use crate::physical_residency::observation::PreparedTracePage;
use crate::{
    PhysicalResidencyRetirementDenial as Denial, RetiredOperationAllocation, RetiredPhysicalFrame,
};
use std::sync::{atomic::Ordering, TryLockError};
use worth_execution::ExecutionRequest;

#[cfg(test)]
mod tests;

impl PoolInner {
    pub(crate) fn retire_frame(
        &self,
        held: &mut Option<PhysicalFrameLease>,
        request: &ExecutionRequest<'_, '_>,
        page: &mut PreparedTracePage,
    ) -> Result<RetiredPhysicalFrame, Denial> {
        let mut state = self.try_retirement_state()?;
        let lease = held.as_ref().expect("original frame custody");
        let entry = state
            .frames
            .get(&lease.key.coordinate)
            .ok_or(Denial::FrameIdentityMismatch)?;
        let FrameState::Resident(bytes) = &entry.state else {
            return Err(Denial::FrameIdentityMismatch);
        };
        if !lease.armed
            || entry.pins == 0
            || !Arc::ptr_eq(bytes, &lease.bytes)
            || entry.resident_generation != Some(lease.resident_generation)
        {
            return Err(Denial::FrameIdentityMismatch);
        }
        let key = lease.key;
        let last_pin = entry.pins == 1;
        let evictable = last_pin && !entry.dirty;
        state.accounting.check_pin_retirement(last_pin)?;
        let recorder = state.accounting.retirement_recorder();
        request.consult().map_err(Denial::Execution)?;
        let events = recorder.prepare_pin_release(last_pin, page)?;
        let retired = RetiredPhysicalFrame {
            _owner: Arc::clone(&lease.owner),
        };

        // No fallible work follows. Resident bytes and this temporary original
        // pool owner remain held while the local frame shares are relinquished.
        state
            .frames
            .get_mut(&key.coordinate)
            .expect("validated resident")
            .pins -= 1;
        state.accounting.release_pin_counters(last_pin);
        if evictable {
            state.append_evictable(key.coordinate);
        }
        let mut lease = held.take().expect("original frame custody");
        lease.armed = false;
        events.commit();
        drop(lease);
        self.changed.notify_all();
        Ok(retired)
    }

    pub(crate) fn retire_operation(
        &self,
        held: &mut Option<OperationAllocationGrant>,
        request: &ExecutionRequest<'_, '_>,
        page: &mut PreparedTracePage,
    ) -> Result<RetiredOperationAllocation, Denial> {
        let mut state = self.try_retirement_state()?;
        let grant = held.as_ref().expect("original operation custody");
        if grant.active_use_bytes.load(Ordering::Acquire) != 0 {
            return Err(Denial::ActiveAllocationUse);
        }
        state
            .accounting
            .check_operation_retirement(grant.scope, grant.bytes)?;
        let recorder = state.accounting.retirement_recorder();
        request.consult().map_err(Denial::Execution)?;
        let events = recorder.prepare_operation_release(grant.scope, grant.bytes, page)?;
        let retired = RetiredOperationAllocation {
            _owner: Arc::clone(&grant.owner),
        };

        state
            .accounting
            .release_operation_counters(grant.scope, grant.bytes);
        let mut grant = held.take().expect("original operation custody");
        grant.armed = false;
        events.commit();
        drop(grant);
        self.changed.notify_all();
        Ok(retired)
    }

    fn try_retirement_state(&self) -> Result<MutexGuard<'_, PoolState>, Denial> {
        let state = self.state.try_lock().map_err(|cause| match cause {
            TryLockError::WouldBlock => Denial::Busy,
            TryLockError::Poisoned(_) => Denial::PoisonedOwner,
        })?;
        if state.closed {
            return Err(Denial::PoolClosed);
        }
        Ok(state)
    }
}
