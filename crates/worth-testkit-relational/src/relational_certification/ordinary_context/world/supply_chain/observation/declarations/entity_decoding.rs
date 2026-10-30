use super::*;

pub fn decode_entity(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
) -> Result<EntityRecord, ObservationError> {
    match key.kind {
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(PortRecord {
            code: u16_value(key, state, "port_code")?,
            name: string_value(key, state, "name")?,
            region: debug_region(key, state, "region")?,
            posture: debug_posture(key, state, "posture")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(TerminalRecord {
            name: string_value(key, state, "name")?,
            capacity: crate::relational_certification::ordinary_context::world::supply_chain::schema::Capacity(u32_value(key, state, "capacity")?),
            posture: debug_posture(key, state, "posture")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(crate::relational_certification::ordinary_context::world::supply_chain::schema::BerthRecord {
            name: string_value(key, state, "name")?,
            depth: crate::relational_certification::ordinary_context::world::supply_chain::schema::DepthMeters(u16_value(key, state, "depth")?),
            capacity: crate::relational_certification::ordinary_context::world::supply_chain::schema::Capacity(u32_value(key, state, "capacity")?),
            posture: debug_posture(key, state, "posture")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(VesselRecord {
            call_sign: string_value(key, state, "call_sign")?,
            class: debug_class(key, state, "class")?,
            capacity: crate::relational_certification::ordinary_context::world::supply_chain::schema::Capacity(u32_value(key, state, "capacity")?),
            posture: debug_posture(key, state, "posture")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(VoyageRecord {
            status: debug_status(key, state, "status")?,
            departure: crate::relational_certification::ordinary_context::world::supply_chain::schema::Minute(u32_value(key, state, "departure")?),
            arrival: crate::relational_certification::ordinary_context::world::supply_chain::schema::Minute(u32_value(key, state, "arrival")?),
            revision: u16_value(key, state, "revision")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(PortCallRecord {
            sequence: u16_value(key, state, "sequence")?,
            revision: u16_value(key, state, "revision")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(CargoLotRecord {
            mass: crate::relational_certification::ordinary_context::world::supply_chain::schema::MassTons(u32_value(key, state, "mass")?),
            customer_code: crate::relational_certification::ordinary_context::world::supply_chain::schema::CargoCode(string_value(key, state, "customer_code")?),
            hazard: debug_hazard(key, state, "hazard")?,
            booking: debug_booking(key, state, "booking")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Inspection(InspectionRecord {
            result: debug_result(key, state, "result")?,
            minute: crate::relational_certification::ordinary_context::world::supply_chain::schema::Minute(u32_value(key, state, "minute")?),
        })),
    }
}

pub fn value<'a>(
    key: EntityKey,
    state: &'a AuthoritativeRecordAspectState,
    name: &str,
) -> Result<&'a AspectValue, ObservationError> {
    let aspect = worth_foundational::facade::AspectKey::new(name)
        .expect("canonical Supply Chain aspect key");
    let Some(entry) = state.get(&aspect) else {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::MissingAspect {
            entity: key,
            name: name.to_owned(),
        });
    };
    match entry.view() {
        worth_foundational::facade::ContractValidatedAspectValueView::Scalar(value) => Ok(value),
        worth_foundational::facade::ContractValidatedAspectValueView::Struct(_) => Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::InvalidAspect {
            entity: key,
            name: name.to_owned(),
            detail: "expected scalar value".to_owned(),
        }),
    }
}

pub fn string_value(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<String, ObservationError> {
    match value(key, state, name)? {
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::Raw(value),
        ) => Ok(value.clone()),
        _ => Err(invalid(key, name, "expected raw string")),
    }
}

pub fn u16_value(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<u16, ObservationError> {
    match value(key, state, name)? {
        worth_foundational::facade::AspectValue::UInt64(value) => (*value)
            .try_into()
            .map_err(|_| invalid(key, name, "UInt64 value exceeds u16")),
        _ => Err(invalid(key, name, "expected UInt64")),
    }
}

pub fn u32_value(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<u32, ObservationError> {
    match value(key, state, name)? {
        worth_foundational::facade::AspectValue::UInt64(value) => (*value)
            .try_into()
            .map_err(|_| invalid(key, name, "UInt64 value exceeds u32")),
        _ => Err(invalid(key, name, "expected UInt64")),
    }
}

pub fn invalid(key: EntityKey, name: &str, detail: &str) -> ObservationError {
    crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::InvalidAspect {
        entity: key,
        name: name.to_owned(),
        detail: detail.to_owned(),
    }
}
