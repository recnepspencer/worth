pub use {sha2::Digest, sha2::Sha256};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass, crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture, crate::relational_certification::ordinary_context::world::supply_chain::schema::Region, crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind};

pub fn canonical_bytes(observation: &ExpectedSupplyChainObservation) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"supply-chain-expected-v1\0");
    write_u16(&mut bytes, observation.schema.version.major);
    write_u16(&mut bytes, observation.schema.version.minor);
    write_u8(
        &mut bytes,
        match observation.schema.version.hazard {
            crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardSchema::V1 => 1,
            crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardSchema::V2 => 2,
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

pub fn digest(observation: &ExpectedSupplyChainObservation) -> [u8; 32] {
    sha2::Sha256::digest(canonical_bytes(observation)).into()
}

pub fn write_u8(bytes: &mut Vec<u8>, value: u8) {
    bytes.push(value);
}

pub fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

pub fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

pub fn write_text(bytes: &mut Vec<u8>, value: &str) {
    write_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value.as_bytes());
}

pub fn write_entity_key(bytes: &mut Vec<u8>, key: EntityKey) {
    write_u8(bytes, entity_tag(key.kind));
    write_u32(bytes, key.ordinal);
}

pub fn write_relation_key(bytes: &mut Vec<u8>, key: RelationKey) {
    write_u8(bytes, relation_tag(key.kind));
    write_u32(bytes, key.ordinal);
}

pub fn entity_tag(kind: EntityKind) -> u8 {
    match kind {
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port => 1,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal => 2,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth => 3,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel => 4,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage => 5,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall => 6,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot => 7,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection => 8,
    }
}

pub fn relation_tag(kind: RelationKind) -> u8 {
    match kind {
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort => 1,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::BerthAtTerminal => 2,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth => 3,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageUsesVessel => 4,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall => 5,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort => 6,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes => 7,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage => 8,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::InspectionCoversVessel => 9,
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone => 10,
    }
}

pub fn write_entity(bytes: &mut Vec<u8>, record: &EntityRecord) {
    match record {
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(value) => {
            write_u16(bytes, value.code);
            write_text(bytes, &value.name);
            match value.region {
                crate::relational_certification::ordinary_context::world::supply_chain::schema::Region::NorthReach => write_u8(bytes, 1),
                crate::relational_certification::ordinary_context::world::supply_chain::schema::Region::SouthReach => write_u8(bytes, 2),
                crate::relational_certification::ordinary_context::world::supply_chain::schema::Region::Generated(index) => {
                    write_u8(bytes, 3);
                    write_u16(bytes, index);
                }
            }
            write_u8(bytes, posture_tag(value.posture));
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(value) => {
            write_text(bytes, &value.name);
            write_u32(bytes, value.capacity.0);
            write_u8(bytes, posture_tag(value.posture));
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(value) => {
            write_text(bytes, &value.name);
            write_u16(bytes, value.depth.0);
            write_u32(bytes, value.capacity.0);
            write_u8(bytes, posture_tag(value.posture));
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(value) => {
            write_text(bytes, &value.call_sign);
            write_u8(bytes, value.class as u8);
            write_u32(bytes, value.capacity.0);
            write_u8(bytes, posture_tag(value.posture));
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(value) => {
            write_u8(
                bytes,
                match value.status {
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Planned => 1,
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Delayed => 2,
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Rerouted => 3,
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Held => 4,
                },
            );
            write_u32(bytes, value.departure.0);
            write_u32(bytes, value.arrival.0);
            write_u16(bytes, value.revision);
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(value) => {
            write_u16(bytes, value.sequence);
            write_u16(bytes, value.revision);
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(value) => {
            write_u32(bytes, value.mass.0);
            write_text(bytes, &value.customer_code.0);
            write_u8(
                bytes,
                match value.hazard {
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::General => 1,
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::Medical => 2,
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::Industrial => 3,
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::HazardousV2 => 4,
                },
            );
            write_u8(bytes, value.booking as u8);
        }
        crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Inspection(value) => {
            write_u8(bytes, value.result as u8);
            write_u32(bytes, value.minute.0);
        }
    }
}

pub fn posture_tag(posture: OperatingPosture) -> u8 {
    match posture {
        crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Open => 1,
        crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Maintenance => 2,
        crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Retired => 3,
    }
}

pub fn write_branch(bytes: &mut Vec<u8>, branch: BranchLabel) {
    write_u8(bytes, branch as u8);
}

pub fn write_delta(bytes: &mut Vec<u8>, delta: DeltaId) {
    write_u8(bytes, delta as u8);
}