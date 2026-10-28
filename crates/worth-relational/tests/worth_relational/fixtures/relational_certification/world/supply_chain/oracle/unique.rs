pub(crate) use super::super::schema::{
    Capacity, EntityRecord, OperatingPosture, VesselClass, VesselRecord,
};
pub(crate) use super::super::semantic_key::{EntityKey, EntityKind};
pub(crate) use super::ancestry::OracleBranch;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UniqueEntityFieldOracleError {
    DuplicateValue(String),
    KeyAlreadyPresent(EntityKey),
}

pub(crate) fn insert_vessel(
    branch: &OracleBranch,
    key: EntityKey,
    call_sign: &str,
) -> Result<OracleBranch, UniqueEntityFieldOracleError> {
    if branch.state.entity(key).is_some() {
        return Err(UniqueEntityFieldOracleError::KeyAlreadyPresent(key));
    }
    if branch.state.entities.values().any(|record| {
        matches!(
            record,
            EntityRecord::Vessel(VesselRecord { call_sign: observed, .. })
                if observed == call_sign
        )
    }) {
        return Err(UniqueEntityFieldOracleError::DuplicateValue(
            call_sign.to_owned(),
        ));
    }

    let mut next = branch.clone();
    next.state = next.state.replace_entity(
        key,
        EntityRecord::Vessel(VesselRecord {
            call_sign: call_sign.to_owned(),
            class: VesselClass::Feeder,
            capacity: Capacity(9_999),
            posture: OperatingPosture::Open,
        }),
    );
    Ok(next)
}

pub(crate) fn vessel_call_signs(branch: &OracleBranch) -> Vec<String> {
    branch
        .state
        .entities
        .values()
        .filter_map(|record| match record {
            EntityRecord::Vessel(vessel) => Some(vessel.call_sign.clone()),
            EntityRecord::Port(_)
            | EntityRecord::Terminal(_)
            | EntityRecord::Berth(_)
            | EntityRecord::Voyage(_)
            | EntityRecord::PortCall(_)
            | EntityRecord::CargoLot(_)
            | EntityRecord::Inspection(_) => None,
        })
        .collect()
}

pub(crate) fn next_vessel_key(branch: &OracleBranch) -> EntityKey {
    let ordinal = branch
        .state
        .entities
        .keys()
        .filter(|key| key.kind == EntityKind::Vessel)
        .map(|key| key.ordinal)
        .max()
        .map_or(0, |ordinal| ordinal + 1);
    EntityKey::new(EntityKind::Vessel, ordinal)
}
