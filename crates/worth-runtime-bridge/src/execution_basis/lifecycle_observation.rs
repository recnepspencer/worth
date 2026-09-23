use std::sync::Arc;

use worth_signal::facade::{
    ResourceInFlightStatus, ResourceQueuePressureObservation, ResourceRequestHandle,
};

use crate::source::{with_async_request_signal_runtime, BridgeSignalRuntimeCustody};

use super::reservation::{
    BridgeExecutionBasisReservationKey, BridgeExecutionBasisReservationRegistry,
};
use super::BridgeBoundExecutionBasis;

#[derive(Clone)]
pub struct BridgeExecutionBasisLifecycleObserver {
    bridge_runtime_key: u64,
    request: ResourceRequestHandle,
    reservations: Arc<BridgeExecutionBasisReservationRegistry>,
    reservation: BridgeExecutionBasisReservationKey,
    // Keep a Bridge-owned runtime alive through the observer's last field drop.
    _runtime_custody: Option<BridgeSignalRuntimeCustody>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeExecutionBasisLifecycleObservation {
    reservation_active: bool,
    signal_status: Option<BridgeExecutionBasisLifecycleSignalStatus>,
    managed_queue_pressure: Option<ResourceQueuePressureObservation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeExecutionBasisLifecycleSignalStatus {
    Active,
    Fulfilled,
    Rejected,
    Superseded,
    Cancelled,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeExecutionBasisLifecycleObservationFailureKind {
    SignalRuntimeThreadAffinityViolation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeExecutionBasisLifecycleObservationFailure {
    kind: BridgeExecutionBasisLifecycleObservationFailureKind,
    detail: String,
}

impl BridgeBoundExecutionBasis {
    pub fn lifecycle_observer(&self) -> BridgeExecutionBasisLifecycleObserver {
        let reservation = self
            .reservation
            .as_ref()
            .expect("active bound execution basis retains its reservation");
        let (reservations, reservation) = reservation.observer_parts();
        BridgeExecutionBasisLifecycleObserver {
            bridge_runtime_key: self.bridge_runtime_key,
            request: self.request.request_handle(),
            reservations,
            reservation,
            _runtime_custody: self.request.lifecycle_observation_custody(),
        }
    }
}

impl BridgeExecutionBasisLifecycleObserver {
    pub fn observe(
        &self,
    ) -> Result<
        BridgeExecutionBasisLifecycleObservation,
        BridgeExecutionBasisLifecycleObservationFailure,
    > {
        let reservation_active = self.reservations.contains(&self.reservation);
        let (signal_status, managed_queue_pressure) =
            with_async_request_signal_runtime(self.bridge_runtime_key, |runtime| {
                runtime
                    .in_flight_resource_request(self.request)
                    .map(|request| {
                        (
                            Some(project_signal_status(request.status())),
                            request.managed_queue_pressure(),
                        )
                    })
                    .unwrap_or((None, None))
            })
            .map_err(|error| {
                BridgeExecutionBasisLifecycleObservationFailure {
                kind: BridgeExecutionBasisLifecycleObservationFailureKind::
                    SignalRuntimeThreadAffinityViolation,
                detail: format!(
                    "bridge Signal runtime {} belongs to thread {:?}, not {:?}",
                    error.runtime_key(),
                    error.owner(),
                    error.current()
                ),
            }
            })?;
        Ok(BridgeExecutionBasisLifecycleObservation {
            reservation_active,
            signal_status,
            managed_queue_pressure,
        })
    }
}

impl BridgeExecutionBasisLifecycleObservation {
    pub const fn reservation_active(self) -> bool {
        self.reservation_active
    }

    pub const fn signal_status(self) -> Option<BridgeExecutionBasisLifecycleSignalStatus> {
        self.signal_status
    }

    pub const fn managed_queue_pressure(self) -> Option<ResourceQueuePressureObservation> {
        self.managed_queue_pressure
    }
}

const fn project_signal_status(
    status: ResourceInFlightStatus,
) -> BridgeExecutionBasisLifecycleSignalStatus {
    match status {
        ResourceInFlightStatus::Active => BridgeExecutionBasisLifecycleSignalStatus::Active,
        ResourceInFlightStatus::Fulfilled => BridgeExecutionBasisLifecycleSignalStatus::Fulfilled,
        ResourceInFlightStatus::Rejected => BridgeExecutionBasisLifecycleSignalStatus::Rejected,
        ResourceInFlightStatus::Superseded => BridgeExecutionBasisLifecycleSignalStatus::Superseded,
        ResourceInFlightStatus::Cancelled => BridgeExecutionBasisLifecycleSignalStatus::Cancelled,
        ResourceInFlightStatus::TimedOut => BridgeExecutionBasisLifecycleSignalStatus::TimedOut,
    }
}

impl BridgeExecutionBasisLifecycleObservationFailure {
    pub const fn kind(&self) -> BridgeExecutionBasisLifecycleObservationFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}
