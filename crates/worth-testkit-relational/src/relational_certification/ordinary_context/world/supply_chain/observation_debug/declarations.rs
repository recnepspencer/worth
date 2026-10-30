pub use worth_foundational::facade::AuthoritativeRecordAspectState;

pub use crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey;
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::observation::invalid,
    crate::relational_certification::ordinary_context::world::supply_chain::observation::string_value,
    crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError,
};
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus,
    crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass,
    crate::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult,
    crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture,
    crate::relational_certification::ordinary_context::world::supply_chain::schema::Region,
    crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselClass,
    crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus,
};

pub fn debug_text(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<String, ObservationError> {
    string_value(key, state, name)
}

pub fn debug_region(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<Region, ObservationError> {
    match debug_text(key, state, name)?.as_str() {
        "NorthReach" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::Region::NorthReach),
        "SouthReach" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::Region::SouthReach),
        other => other
            .strip_prefix("Generated(")
            .and_then(|v| v.strip_suffix(')'))
            .and_then(|v| v.parse().ok())
            .map(crate::relational_certification::ordinary_context::world::supply_chain::schema::Region::Generated)
            .ok_or_else(|| invalid(key, name, "unknown region")),
    }
}

pub fn debug_posture(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<OperatingPosture, ObservationError> {
    match debug_text(key, state, name)?.as_str() {
        "Open" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Open),
        "Maintenance" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Maintenance),
        "Retired" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Retired),
        _ => Err(invalid(key, name, "unknown operating posture")),
    }
}

pub fn debug_class(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<VesselClass, ObservationError> {
    match debug_text(key, state, name)?.as_str() {
        "Feeder" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselClass::Feeder),
        "Panamax" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselClass::Panamax),
        "HeavyLift" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselClass::HeavyLift),
        _ => Err(invalid(key, name, "unknown vessel class")),
    }
}

pub fn debug_status(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<VoyageStatus, ObservationError> {
    match debug_text(key, state, name)?.as_str() {
        "Planned" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Planned),
        "Delayed" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Delayed),
        "Rerouted" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Rerouted),
        "Held" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Held),
        _ => Err(invalid(key, name, "unknown voyage status")),
    }
}

pub fn debug_hazard(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<HazardClass, ObservationError> {
    match debug_text(key, state, name)?.as_str() {
        "General" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::General),
        "Medical" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::Medical),
        "Industrial" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::Industrial),
        "HazardousV2" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::HazardousV2),
        _ => Err(invalid(key, name, "unknown hazard class")),
    }
}

pub fn debug_booking(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<BookingStatus, ObservationError> {
    match debug_text(key, state, name)?.as_str() {
        "Available" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus::Available),
        "Booked" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus::Booked),
        "Held" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus::Held),
        _ => Err(invalid(key, name, "unknown booking status")),
    }
}

pub fn debug_result(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<InspectionResult, ObservationError> {
    match debug_text(key, state, name)?.as_str() {
        "Pending" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult::Pending),
        "Passed" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult::Passed),
        "Flagged" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult::Flagged),
        _ => Err(invalid(key, name, "unknown inspection result")),
    }
}
