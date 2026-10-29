pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::Capacity, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture, crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselClass, crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselRecord};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UniqueEntityFieldOracleError {
    DuplicateValue(String),
    KeyAlreadyPresent(EntityKey),
}

pub fn insert_vessel(
    branch: &OracleBranch,
    key: EntityKey,
    call_sign: &str,
) -> Result<OracleBranch, UniqueEntityFieldOracleError> {
    if branch.state.entity(key).is_some() {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::unique::UniqueEntityFieldOracleError::KeyAlreadyPresent(key));
    }
    if branch.state.entities.values().any(|record| {
        matches!(
            record,
            crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(VesselRecord { call_sign: observed, .. })
                if observed == call_sign
        )
    }) {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::unique::UniqueEntityFieldOracleError::DuplicateValue(
            call_sign.to_owned(),
        ));
    }

    let mut next = branch.clone();
    next.state = next.state.replace_entity(
        key,
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(VesselRecord {
            call_sign: call_sign.to_owned(),
            class: crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselClass::Feeder,
            capacity: Capacity(9_999),
            posture: crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Open,
        }),
    );
    Ok(next)
}

pub fn vessel_call_signs(branch: &OracleBranch) -> Vec<String> {
    branch
        .state
        .entities
        .values()
        .filter_map(|record| match record {
            crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(vessel) => Some(vessel.call_sign.clone()),
            crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(_)
            | crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(_)
            | crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(_)
            | crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(_)
            | crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(_)
            | crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(_)
            | crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Inspection(_) => None,
        })
        .collect()
}

pub fn next_vessel_key(branch: &OracleBranch) -> EntityKey {
    let ordinal = branch
        .state
        .entities
        .keys()
        .filter(|key| key.kind == crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel)
        .map(|key| key.ordinal)
        .max()
        .map_or(0, |ordinal| ordinal + 1);
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel, ordinal)
}