//! Live serial-caller memory hold tracking beneath an inner narrowed lease.
//!
//! `worth-execution` forbids opening a lease-free scope beneath a leased scope,
//! so `runner.rs` captures a zero-byte serial reservation while still inside
//! the caller's outer serial scope. Subsequent `QueryRequestExecution::reserve`
//! and `resize` calls adjust that live `Held::Serial` ticket directly via
//! `ExecutionMemoryReservation::resize`, charging the caller's exact
//! `SerialMemoryBudget` ledger without opening a nested scope.

use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, MemoryLimitDenial};

use super::super::execution_denial::memory_denial;
use super::advancement::{advancement_denial, WorthQueryAdvancementDenial};
use super::Resource;

pub(super) type SharedCallerSerialHold = Arc<Mutex<CallerSerialHoldState>>;

pub(super) enum CallerSerialHoldState {
    Unbound,
    Bound {
        reservation: ExecutionMemoryReservation,
        bytes: u64,
    },
    Unaccounted,
}

pub(super) fn unbound_hold() -> SharedCallerSerialHold {
    Arc::new(Mutex::new(CallerSerialHoldState::Unbound))
}

/// Captures a zero-byte reservation on the active serial scope (must be called
/// inside the caller's serial scope before opening any inner leased scope).
pub(super) fn bind_in_active_serial_scope(shared: &SharedCallerSerialHold) {
    let mut guard = shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !matches!(*guard, CallerSerialHoldState::Unbound) {
        return;
    }
    *guard = match ExecutionMemoryReservation::reserve_in_scope(None, 0) {
        Ok(reservation) => CallerSerialHoldState::Bound {
            reservation,
            bytes: 0,
        },
        Err(_) => CallerSerialHoldState::Unaccounted,
    };
}

/// Secondary reservation handle keeping the caller's serial memory ledger in
/// lockstep with a `QueryMemoryReservation`.
pub(super) enum SecondarySerialHold {
    Direct(ExecutionMemoryReservation),
    Shared {
        shared: SharedCallerSerialHold,
        bytes: u64,
    },
}

impl SecondarySerialHold {
    pub(super) fn reserve(
        caller: worth_execution::ExecutionRequest<'_, '_>,
        shared: Option<&SharedCallerSerialHold>,
        bytes: u64,
    ) -> Result<Option<Self>, Resource> {
        if let Some(shared) = shared {
            let mut guard = shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match &mut *guard {
                CallerSerialHoldState::Bound {
                    reservation,
                    bytes: total,
                } => {
                    let target = total.checked_add(bytes).ok_or(Resource::CapacityOverflow)?;
                    reservation.resize(target).map_err(memory_denial)?;
                    *total = target;
                    return Ok(Some(Self::Shared {
                        shared: Arc::clone(shared),
                        bytes,
                    }));
                }
                CallerSerialHoldState::Unaccounted => return Ok(None),
                CallerSerialHoldState::Unbound => {}
            }
        }
        caller
            .reserve_retained_memory(bytes)
            .map(|held| held.map(Self::Direct))
            .map_err(|denial| match advancement_denial(denial) {
                WorthQueryAdvancementDenial::Resource(resource) => resource,
                _ => Resource::NoActiveExecutionScope,
            })
    }

    pub(super) fn resize(&mut self, to: u64) -> Result<(), Resource> {
        match self {
            Self::Direct(reservation) => reservation.resize(to).map_err(memory_denial),
            Self::Shared { shared, bytes } => {
                let mut guard = shared
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let CallerSerialHoldState::Bound {
                    reservation,
                    bytes: total,
                } = &mut *guard
                else {
                    return Ok(());
                };
                let base = total.saturating_sub(*bytes);
                let target = base.checked_add(to).ok_or(Resource::CapacityOverflow)?;
                reservation.resize(target).map_err(memory_denial)?;
                *total = target;
                *bytes = to;
                Ok(())
            }
        }
    }

    pub(super) fn rollback_to(&mut self, prev: u64) {
        let _ = self.resize(prev);
    }
}

impl Drop for SecondarySerialHold {
    fn drop(&mut self) {
        if let Self::Shared { shared, bytes } = self
            && *bytes != 0
        {
            let mut guard = shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let CallerSerialHoldState::Bound {
                reservation,
                bytes: total,
            } = &mut *guard
            {
                let next = total.saturating_sub(*bytes);
                let _: Result<(), MemoryLimitDenial> = reservation.resize(next);
                *total = next;
                *bytes = 0;
            }
        }
    }
}
