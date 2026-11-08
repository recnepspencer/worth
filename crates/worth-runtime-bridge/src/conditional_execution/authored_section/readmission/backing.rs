//! Retained cold projections keep the original request's charge through final release.
use super::BridgeAuthoredReadmissionDenial as D;
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(in crate::conditional_execution) struct Backing {
    held: Mutex<Held>,
}
struct Held {
    bytes: u64,
    reservation: Option<ExecutionMemoryReservation>,
}
impl Backing {
    pub(super) fn new(execution: &ExecutionRequest<'_, '_>) -> Result<Arc<Self>, D> {
        let bytes = crate::conditional_execution::retention::arc_charge::<Self>()
            .map_err(|_| D::section(super::super::BridgeAuthoredSectionDenial::SizeLimit))?;
        let reservation = execution.reserve_retained_memory(bytes).map_err(|cause| {
            D::section(super::super::BridgeAuthoredSectionDenial::Request(cause))
        })?;
        Ok(Arc::new(Self {
            held: Mutex::new(Held { bytes, reservation }),
        }))
    }
    pub(in crate::conditional_execution) fn grow(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        bytes: u64,
    ) -> Result<(), D> {
        execution.consult().map_err(|cause| {
            D::section(super::super::BridgeAuthoredSectionDenial::Request(cause))
        })?;
        let mut held = self
            .held
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let total = held.bytes.checked_add(bytes).ok_or(D::section(
            super::super::BridgeAuthoredSectionDenial::SizeLimit,
        ))?;
        if let Some(reservation) = &mut held.reservation {
            reservation.resize(total).map_err(D::Memory)?;
        }
        held.bytes = total;
        Ok(())
    }
}
