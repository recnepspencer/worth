use std::sync::{Arc, Mutex, MutexGuard};

use crate::{IoSchedulerBackendCapabilityAdmission, IoSchedulerSecurityScopeAdmission};

use super::{
    super::{capacity::require_capacity, ForegroundLaneDeclaration, ForegroundResourceBudget},
    admit_physical_instance_foreground_reservation, PhysicalInstanceForegroundAdmissionDenial,
    PhysicalInstanceForegroundAdmissionRequest, PhysicalInstanceForegroundCapacityLease,
    PhysicalInstanceForegroundReservation,
};

#[derive(Clone, Debug)]
pub struct PhysicalInstanceForegroundCapacity {
    state: Arc<Mutex<PhysicalInstanceForegroundCapacityState>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalInstanceForegroundCapacitySnapshot {
    configured: ForegroundResourceBudget,
    available: ForegroundResourceBudget,
    active_reservations: u64,
    admitted_reservations: u64,
    denied_reservations: u64,
    released_reservations: u64,
}

#[derive(Debug)]
pub(super) struct PhysicalInstanceForegroundCapacityState {
    configured: ForegroundResourceBudget,
    available: ForegroundResourceBudget,
    active_reservations: u64,
    admitted_reservations: u64,
    denied_reservations: u64,
    released_reservations: u64,
}

impl PhysicalInstanceForegroundCapacity {
    pub fn new(configured: ForegroundResourceBudget) -> Option<Self> {
        if configured.is_empty() {
            return None;
        }
        Some(Self {
            state: Arc::new(Mutex::new(PhysicalInstanceForegroundCapacityState {
                configured,
                available: configured,
                active_reservations: 0,
                admitted_reservations: 0,
                denied_reservations: 0,
                released_reservations: 0,
            })),
        })
    }

    pub fn reserve(
        &self,
        lane: ForegroundLaneDeclaration,
        backend: &IoSchedulerBackendCapabilityAdmission,
        security: &IoSchedulerSecurityScopeAdmission,
    ) -> Result<PhysicalInstanceForegroundReservation, PhysicalInstanceForegroundAdmissionDenial>
    {
        self.reserve_live(lane, backend, security, false)
    }

    /// Atomically reserves live resources while leaving at least half of every
    /// configured resource dimension available to foreground work. The returned
    /// lease remains charged until settlement or cancellation drops it.
    pub fn reserve_background(
        &self,
        lane: ForegroundLaneDeclaration,
        backend: &IoSchedulerBackendCapabilityAdmission,
        security: &IoSchedulerSecurityScopeAdmission,
    ) -> Result<PhysicalInstanceForegroundReservation, PhysicalInstanceForegroundAdmissionDenial>
    {
        self.reserve_live(lane, backend, security, true)
    }

    fn reserve_live(
        &self,
        lane: ForegroundLaneDeclaration,
        backend: &IoSchedulerBackendCapabilityAdmission,
        security: &IoSchedulerSecurityScopeAdmission,
        background: bool,
    ) -> Result<PhysicalInstanceForegroundReservation, PhysicalInstanceForegroundAdmissionDenial>
    {
        let requested = lane.requested_budget();
        let mut state = lock(&self.state);
        let protected = if background {
            state.configured.half_rounded_up()
        } else {
            ForegroundResourceBudget::new()
        };
        if let Err((denial, _)) = require_capacity(protected, state.available) {
            state.denied_reservations = state.denied_reservations.saturating_add(1);
            return Err(PhysicalInstanceForegroundAdmissionDenial::Foreground(
                denial,
            ));
        }
        let eligible = state
            .available
            .checked_sub(protected)
            .expect("protected capacity checked under lock");
        let receipt = match admit_physical_instance_foreground_reservation(
            PhysicalInstanceForegroundAdmissionRequest::new(lane, backend, security, eligible),
        ) {
            Ok(receipt) => receipt,
            Err(denial) => {
                state.denied_reservations = state.denied_reservations.saturating_add(1);
                return Err(denial);
            }
        };
        state.available = state
            .available
            .checked_sub(requested)
            .expect("admission proved the requested foreground capacity is available");
        state.active_reservations = state.active_reservations.saturating_add(1);
        state.admitted_reservations = state.admitted_reservations.saturating_add(1);
        Ok(PhysicalInstanceForegroundReservation::new(
            receipt,
            PhysicalInstanceForegroundCapacityLease::new(self.state.clone(), requested),
        ))
    }

    pub fn snapshot(&self) -> PhysicalInstanceForegroundCapacitySnapshot {
        let state = lock(&self.state);
        PhysicalInstanceForegroundCapacitySnapshot {
            configured: state.configured,
            available: state.available,
            active_reservations: state.active_reservations,
            admitted_reservations: state.admitted_reservations,
            denied_reservations: state.denied_reservations,
            released_reservations: state.released_reservations,
        }
    }
}

impl PhysicalInstanceForegroundCapacitySnapshot {
    pub const fn configured(self) -> ForegroundResourceBudget {
        self.configured
    }

    pub const fn available(self) -> ForegroundResourceBudget {
        self.available
    }

    pub const fn active_reservations(self) -> u64 {
        self.active_reservations
    }

    pub const fn admitted_reservations(self) -> u64 {
        self.admitted_reservations
    }

    pub const fn denied_reservations(self) -> u64 {
        self.denied_reservations
    }

    pub const fn released_reservations(self) -> u64 {
        self.released_reservations
    }
}

pub(super) fn release(
    state: &Arc<Mutex<PhysicalInstanceForegroundCapacityState>>,
    reserved: ForegroundResourceBudget,
) {
    let mut state = lock(state);
    let available = state
        .available
        .checked_add(reserved)
        .expect("admitted foreground capacity release cannot overflow");
    assert!(
        require_capacity(available, state.configured).is_ok(),
        "foreground capacity release cannot exceed configured ownership"
    );
    state.available = available;
    state.active_reservations = state
        .active_reservations
        .checked_sub(1)
        .expect("a foreground capacity lease releases exactly once");
    state.released_reservations = state.released_reservations.saturating_add(1);
}

fn lock(
    state: &Arc<Mutex<PhysicalInstanceForegroundCapacityState>>,
) -> MutexGuard<'_, PhysicalInstanceForegroundCapacityState> {
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
