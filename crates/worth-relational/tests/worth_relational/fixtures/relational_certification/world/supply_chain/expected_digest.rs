pub(crate) use sha2::{Digest, Sha256};

pub(crate) use super::delta::DeltaId;
pub(crate) use super::expected_observation::ExpectedSupplyChainObservation;
pub(crate) use super::schema::{EntityRecord, HazardClass, OperatingPosture, Region, VoyageStatus};
pub(crate) use super::semantic_key::{
    BranchLabel, EntityKey, EntityKind, RelationKey, RelationKind,
};

pub(crate) fn canonical_bytes(observation: &ExpectedSupplyChainObservation) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"supply-chain-expected-v1\0");
    write_u16(&mut bytes, observation.schema.version.major);
    write_u16(&mut bytes, observation.schema.version.minor);
    write_u8(
        &mut bytes,
        match observation.schema.version.hazard {
            super::schema::HazardSchema::V1 => 1,
            super::schema::HazardSchema::V2 => 2,
        },
    );
    write_u32(&mut bytes, observation.entities.len() as u32);
    for (key, record) in &observation.entities {
        write_entity_key(&mut bytes, *key);
        write_entity(&mut bytes, record);
    }
    write_u32(&mut bytes, observation.relations.len() as u32);
    for (key, edge) in &observation.relations {
        write_relation_key(&mut bytes, *key);
        write_entity_key(&mut bytes, edge.source);
        write_entity_key(&mut bytes, edge.target);
    }
    write_u32(&mut bytes, observation.absent_entities.len() as u32);
    for key in &observation.absent_entities {
        write_entity_key(&mut bytes, *key);
    }
    write_u32(&mut bytes, observation.absent_relations.len() as u32);
    for key in &observation.absent_relations {
        write_relation_key(&mut bytes, *key);
    }
    write_branch(&mut bytes, observation.ancestry.branch);
    match observation.ancestry.parent {
        Some(parent) => {
            write_u8(&mut bytes, 1);
            write_branch(&mut bytes, parent);
        }
        None => write_u8(&mut bytes, 0),
    }
    write_u32(&mut bytes, observation.ancestry.lineage.len() as u32);
    for branch in &observation.ancestry.lineage {
        write_branch(&mut bytes, *branch);
    }
    write_u32(&mut bytes, observation.ancestry.history.len() as u32);
    for event in &observation.ancestry.history {
        write_branch(&mut bytes, event.branch);
        write_delta(&mut bytes, event.delta);
    }
    write_u32(&mut bytes, observation.ancestry.accepted.len() as u32);
    for delta in &observation.ancestry.accepted {
        write_delta(&mut bytes, *delta);
    }
    bytes
}

pub(crate) fn digest(observation: &ExpectedSupplyChainObservation) -> [u8; 32] {
    Sha256::digest(canonical_bytes(observation)).into()
}

pub(crate) fn write_u8(bytes: &mut Vec<u8>, value: u8) {
    bytes.push(value);
}

pub(crate) fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

pub(crate) fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

pub(crate) fn write_text(bytes: &mut Vec<u8>, value: &str) {
    write_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value.as_bytes());
}

pub(crate) fn write_entity_key(bytes: &mut Vec<u8>, key: EntityKey) {
    write_u8(bytes, entity_tag(key.kind));
    write_u32(bytes, key.ordinal);
}

pub(crate) fn write_relation_key(bytes: &mut Vec<u8>, key: RelationKey) {
    write_u8(bytes, relation_tag(key.kind));
    write_u32(bytes, key.ordinal);
}

pub(crate) fn entity_tag(kind: EntityKind) -> u8 {
    match kind {
        EntityKind::Port => 1,
        EntityKind::Terminal => 2,
        EntityKind::Berth => 3,
        EntityKind::Vessel => 4,
        EntityKind::Voyage => 5,
        EntityKind::PortCall => 6,
        EntityKind::CargoLot => 7,
        EntityKind::Inspection => 8,
    }
}

pub(crate) fn relation_tag(kind: RelationKind) -> u8 {
    match kind {
        RelationKind::TerminalAtPort => 1,
        RelationKind::BerthAtTerminal => 2,
        RelationKind::VesselAssignedToBerth => 3,
        RelationKind::VoyageUsesVessel => 4,
        RelationKind::VoyageHasCall => 5,
        RelationKind::CallAtPort => 6,
        RelationKind::CallPrecedes => 7,
        RelationKind::CargoBookedOnVoyage => 8,
        RelationKind::InspectionCoversVessel => 9,
        RelationKind::SharesPilotageZone => 10,
    }
}

pub(crate) fn write_entity(bytes: &mut Vec<u8>, record: &EntityRecord) {
    match record {
        EntityRecord::Port(value) => {
            write_u16(bytes, value.code);
            write_text(bytes, &value.name);
            match value.region {
                Region::NorthReach => write_u8(bytes, 1),
                Region::SouthReach => write_u8(bytes, 2),
                Region::Generated(index) => {
                    write_u8(bytes, 3);
                    write_u16(bytes, index);
                }
            }
            write_u8(bytes, posture_tag(value.posture));
        }
        EntityRecord::Terminal(value) => {
            write_text(bytes, &value.name);
            write_u32(bytes, value.capacity.0);
            write_u8(bytes, posture_tag(value.posture));
        }
        EntityRecord::Berth(value) => {
            write_text(bytes, &value.name);
            write_u16(bytes, value.depth.0);
            write_u32(bytes, value.capacity.0);
            write_u8(bytes, posture_tag(value.posture));
        }
        EntityRecord::Vessel(value) => {
            write_text(bytes, &value.call_sign);
            write_u8(bytes, value.class as u8);
            write_u32(bytes, value.capacity.0);
            write_u8(bytes, posture_tag(value.posture));
        }
        EntityRecord::Voyage(value) => {
            write_u8(
                bytes,
                match value.status {
                    VoyageStatus::Planned => 1,
                    VoyageStatus::Delayed => 2,
                    VoyageStatus::Rerouted => 3,
                    VoyageStatus::Held => 4,
                },
            );
            write_u32(bytes, value.departure.0);
            write_u32(bytes, value.arrival.0);
            write_u16(bytes, value.revision);
        }
        EntityRecord::PortCall(value) => {
            write_u16(bytes, value.sequence);
            write_u16(bytes, value.revision);
        }
        EntityRecord::CargoLot(value) => {
            write_u32(bytes, value.mass.0);
            write_text(bytes, &value.customer_code.0);
            write_u8(
                bytes,
                match value.hazard {
                    HazardClass::General => 1,
                    HazardClass::Medical => 2,
                    HazardClass::Industrial => 3,
                    HazardClass::HazardousV2 => 4,
                },
            );
            write_u8(bytes, value.booking as u8);
        }
        EntityRecord::Inspection(value) => {
            write_u8(bytes, value.result as u8);
            write_u32(bytes, value.minute.0);
        }
    }
}

pub(crate) fn posture_tag(posture: OperatingPosture) -> u8 {
    match posture {
        OperatingPosture::Open => 1,
        OperatingPosture::Maintenance => 2,
        OperatingPosture::Retired => 3,
    }
}

pub(crate) fn write_branch(bytes: &mut Vec<u8>, branch: BranchLabel) {
    write_u8(bytes, branch as u8);
}

pub(crate) fn write_delta(bytes: &mut Vec<u8>, delta: DeltaId) {
    write_u8(bytes, delta as u8);
}
