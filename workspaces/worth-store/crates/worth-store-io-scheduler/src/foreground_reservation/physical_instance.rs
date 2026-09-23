use crate::{IoSchedulerBackendCapabilityAdmission, IoSchedulerSecurityScopeAdmission};

use super::resource_contract::{require_declared_resource_budget, require_lane_resource_contract};
use super::{
    capacity::require_capacity, ForegroundArbitrationDeclaration, ForegroundLaneDeclaration,
    ForegroundLatencyEnvelopeKind, ForegroundReservationAdmissionDenial,
    ForegroundReservationBackendBasis, ForegroundReservationCounterSnapshot,
    ForegroundReservationReceipt, ForegroundResourceBudget,
};

mod capacity;
mod lease;

pub use capacity::{
    PhysicalInstanceForegroundCapacity, PhysicalInstanceForegroundCapacitySnapshot,
};
pub use lease::{PhysicalInstanceForegroundCapacityLease, PhysicalInstanceForegroundReservation};

/// Admission inputs owned by one already-qualified physical Store instance.
///
/// This basis deliberately does not carry or claim the later physical-isolation
/// readiness proof. It admits only the bounded foreground resources that the
/// current physical instance can name at C.5.
#[derive(Debug)]
pub struct PhysicalInstanceForegroundAdmissionRequest<'a> {
    lane: ForegroundLaneDeclaration,
    backend: &'a IoSchedulerBackendCapabilityAdmission,
    security: &'a IoSchedulerSecurityScopeAdmission,
    available: ForegroundResourceBudget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysicalInstanceForegroundAdmissionDenial {
    Foreground(ForegroundReservationAdmissionDenial),
}

impl<'a> PhysicalInstanceForegroundAdmissionRequest<'a> {
    pub const fn new(
        lane: ForegroundLaneDeclaration,
        backend: &'a IoSchedulerBackendCapabilityAdmission,
        security: &'a IoSchedulerSecurityScopeAdmission,
        available: ForegroundResourceBudget,
    ) -> Self {
        Self {
            lane,
            backend,
            security,
            available,
        }
    }
}

pub fn admit_physical_instance_foreground_reservation(
    request: PhysicalInstanceForegroundAdmissionRequest<'_>,
) -> Result<ForegroundReservationReceipt, PhysicalInstanceForegroundAdmissionDenial> {
    let lane = request.lane;
    let envelope = lane
        .envelope()
        .ok_or(ForegroundReservationAdmissionDenial::MissingLaneEnvelope)
        .map_err(PhysicalInstanceForegroundAdmissionDenial::Foreground)?;
    if envelope.kind() == ForegroundLatencyEnvelopeKind::CertificationOnlyTarget {
        return Err(PhysicalInstanceForegroundAdmissionDenial::Foreground(
            ForegroundReservationAdmissionDenial::CertificationOnlyEnvelopeCannotExecute,
        ));
    }
    if envelope.claims_unprovided_service_time() {
        return Err(PhysicalInstanceForegroundAdmissionDenial::Foreground(
            ForegroundReservationAdmissionDenial::UnsupportedServiceTimeEnvelope {
                kind: envelope.kind(),
            },
        ));
    }
    require_declared_resource_budget(lane.requested_budget())
        .map_err(PhysicalInstanceForegroundAdmissionDenial::Foreground)?;
    require_lane_resource_contract(lane.lane(), lane.requested_budget())
        .map_err(PhysicalInstanceForegroundAdmissionDenial::Foreground)?;
    if !lane.backend_requirement_is_store_owned() {
        return Err(PhysicalInstanceForegroundAdmissionDenial::Foreground(
            ForegroundReservationAdmissionDenial::LaneBackendRequirementNotStoreOwned {
                lane: lane.lane(),
                backend_requirement: lane.backend_requirement(),
            },
        ));
    }
    if lane.backend_requirement() != request.backend.requirement() {
        return Err(PhysicalInstanceForegroundAdmissionDenial::Foreground(
            ForegroundReservationAdmissionDenial::LaneBackendRequirementMismatch {
                lane_required: lane.backend_requirement(),
                admitted: request.backend.requirement(),
            },
        ));
    }
    if lane.backend_requirement() == crate::IoSchedulerBackendCapabilityRequirement::SecureFrameIo
        && !request.backend.security_scope_bound()
    {
        return Err(PhysicalInstanceForegroundAdmissionDenial::Foreground(
            ForegroundReservationAdmissionDenial::SecureFrameBackendWasNotSecurityBound,
        ));
    }
    require_capacity(lane.requested_budget(), request.available)
        .map_err(|(denial, _)| PhysicalInstanceForegroundAdmissionDenial::Foreground(denial))?;
    let counters = ForegroundReservationCounterSnapshot::admitted(
        lane.requested_budget(),
        request.available,
        lane.requested_budget(),
    );
    Ok(ForegroundReservationReceipt::admitted(
        lane.lane(),
        ForegroundReservationBackendBasis::new(
            lane.backend_requirement(),
            request.backend.profile(),
            request.backend.evidence_class(),
        ),
        envelope,
        ForegroundArbitrationDeclaration::for_lane(lane.lane()),
        counters,
        request.security.permission().identity(),
    ))
}
