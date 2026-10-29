pub use {std::collections::BTreeMap};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::BerthRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus, crate::relational_certification::ordinary_context::world::supply_chain::schema::Capacity, crate::relational_certification::ordinary_context::world::supply_chain::schema::CargoCode, crate::relational_certification::ordinary_context::world::supply_chain::schema::CargoLotRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::DepthMeters, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass, crate::relational_certification::ordinary_context::world::supply_chain::schema::InspectionRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult, crate::relational_certification::ordinary_context::world::supply_chain::schema::MassTons, crate::relational_certification::ordinary_context::world::supply_chain::schema::Minute, crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture, crate::relational_certification::ordinary_context::world::supply_chain::schema::PortCallRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::PortRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::Region, crate::relational_certification::ordinary_context::world::supply_chain::schema::TerminalRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselClass, crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind};

pub fn build(scale: SupplyChainScale) -> BTreeMap<EntityKey, EntityRecord> {
    let mut entities = std::collections::BTreeMap::new();
    add_ports(scale, &mut entities);
    add_terminals(scale, &mut entities);
    add_berths(scale, &mut entities);
    add_vessels(scale, &mut entities);
    add_voyages(scale, &mut entities);
    add_port_calls(scale, &mut entities);
    add_cargo(scale, &mut entities);
    add_inspections(scale, &mut entities);
    entities
}

pub fn add_ports(scale: SupplyChainScale, entities: &mut BTreeMap<EntityKey, EntityRecord>) {
    for ordinal in 0..scale.ports {
        let region = match scale.region_index(ordinal) {
            0 => crate::relational_certification::ordinary_context::world::supply_chain::schema::Region::NorthReach,
            1 => crate::relational_certification::ordinary_context::world::supply_chain::schema::Region::SouthReach,
            index => crate::relational_certification::ordinary_context::world::supply_chain::schema::Region::Generated(index as u16),
        };
        let name = match ordinal {
            0 => "Meridian".to_owned(),
            1 => "Southpoint".to_owned(),
            _ => format!("port-{:04x}-{ordinal}", scale.seeded(ordinal) as u16),
        };
        entities.insert(
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, ordinal as u32),
            crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(PortRecord {
                code: 1000
                    + ordinal as u16
                    + if ordinal > 1 {
                        (scale.seeded(ordinal) % 7) as u16
                    } else {
                        0
                    },
                name,
                region,
                posture: crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Open,
            }),
        );
    }
}

pub fn add_terminals(
    scale: SupplyChainScale,
    entities: &mut BTreeMap<EntityKey, EntityRecord>,
) {
    for ordinal in 0..scale.terminals {
        let name = if ordinal == 0 {
            "Meridian Container".to_owned()
        } else if ordinal == 1 {
            "Southpoint Container".to_owned()
        } else {
            format!("terminal-{:04x}-{ordinal}", scale.seeded(ordinal) as u16)
        };
        entities.insert(
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal, ordinal as u32),
            crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(TerminalRecord {
                name,
                capacity: Capacity(
                    10_000
                        + ordinal as u32 * 100
                        + if ordinal > 1 {
                            (scale.seeded(ordinal) % 500) as u32
                        } else {
                            0
                        },
                ),
                posture: crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Open,
            }),
        );
    }
}

pub fn add_berths(
    scale: SupplyChainScale,
    entities: &mut BTreeMap<EntityKey, EntityRecord>,
) {
    for ordinal in 0..scale.berths {
        let name = match ordinal {
            0 => "Atlas".to_owned(),
            1 => "Beacon".to_owned(),
            2 => "Southpoint Berth".to_owned(),
            _ => format!("berth-{:04x}-{ordinal}", scale.seeded(ordinal) as u16),
        };
        entities.insert(
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, ordinal as u32),
            crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(BerthRecord {
                name,
                depth: DepthMeters(12 + (ordinal % 4) as u16),
                capacity: Capacity(
                    2_000
                        + ordinal as u32 * 10
                        + if ordinal > 2 {
                            (scale.seeded(ordinal) % 100) as u32
                        } else {
                            0
                        },
                ),
                posture: crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Open,
            }),
        );
    }
}

pub fn add_vessels(
    scale: SupplyChainScale,
    entities: &mut BTreeMap<EntityKey, EntityRecord>,
) {
    for ordinal in 0..scale.vessels {
        let call_sign = if ordinal == 0 {
            "AURORA".to_owned()
        } else {
            format!("VESSEL-{:04x}-{ordinal:03}", scale.seeded(ordinal) as u16)
        };
        entities.insert(
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel, ordinal as u32),
            crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(VesselRecord {
                call_sign,
                class: if ordinal == 2 {
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselClass::HeavyLift
                } else if ordinal % 3 == 0 {
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselClass::Panamax
                } else {
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselClass::Feeder
                },
                capacity: Capacity(
                    4_000
                        + ordinal as u32 * 25
                        + if ordinal > 0 {
                            (scale.seeded(ordinal) % 250) as u32
                        } else {
                            0
                        },
                ),
                posture: crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Open,
            }),
        );
    }
}

pub fn add_voyages(
    scale: SupplyChainScale,
    entities: &mut BTreeMap<EntityKey, EntityRecord>,
) {
    for ordinal in 0..scale.voyages {
        entities.insert(
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, ordinal as u32),
            crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(VoyageRecord {
                status: if ordinal == 1 {
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Held
                } else {
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Planned
                },
                departure: Minute(
                    100 + ordinal as u32 * 10
                        + if ordinal > 0 {
                            (scale.seeded(ordinal) % 5) as u32
                        } else {
                            0
                        },
                ),
                arrival: Minute(
                    200 + ordinal as u32 * 10
                        + if ordinal > 0 {
                            (scale.seeded(ordinal) % 5) as u32
                        } else {
                            0
                        },
                ),
                revision: 0,
            }),
        );
    }
}

pub fn add_port_calls(
    scale: SupplyChainScale,
    entities: &mut BTreeMap<EntityKey, EntityRecord>,
) {
    for ordinal in 0..scale.port_calls {
        entities.insert(
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, ordinal as u32),
            crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(PortCallRecord {
                sequence: (ordinal % 3) as u16,
                revision: 0,
            }),
        );
    }
}

pub fn add_cargo(scale: SupplyChainScale, entities: &mut BTreeMap<EntityKey, EntityRecord>) {
    for ordinal in 0..scale.cargo_lots {
        let (hazard, booking) = match ordinal {
            0 => (crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::Medical, crate::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus::Booked),
            1 => (crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::Industrial, crate::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus::Booked),
            _ => (crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::General, crate::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus::Available),
        };
        entities.insert(
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, ordinal as u32),
            crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(CargoLotRecord {
                mass: MassTons(
                    10 + (ordinal % 20) as u32
                        + if ordinal > 1 {
                            (scale.seeded(ordinal) % 3) as u32
                        } else {
                            0
                        },
                ),
                customer_code: CargoCode(match ordinal {
                    0 => "CARGO-MEDICAL-0000".to_owned(),
                    1 => "CARGO-MACHINE-0001".to_owned(),
                    _ => format!("CARGO-{:016x}-{ordinal:08}", scale.seeded(ordinal)),
                }),
                hazard,
                booking,
            }),
        );
    }
}

pub fn add_inspections(
    scale: SupplyChainScale,
    entities: &mut BTreeMap<EntityKey, EntityRecord>,
) {
    for ordinal in 0..scale.vessels {
        entities.insert(
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection, ordinal as u32),
            crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Inspection(InspectionRecord {
                result: if ordinal == 0 {
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult::Passed
                } else {
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult::Pending
                },
                minute: Minute(150 + ordinal as u32),
            }),
        );
    }
}