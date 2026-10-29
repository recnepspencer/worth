pub(crate) use std::collections::BTreeMap;

pub(crate) use super::scale::SupplyChainScale;
pub(crate) use super::schema::{
    BerthRecord, BookingStatus, Capacity, CargoCode, CargoLotRecord, DepthMeters, EntityRecord,
    HazardClass, InspectionRecord, InspectionResult, MassTons, Minute, OperatingPosture,
    PortCallRecord, PortRecord, Region, TerminalRecord, VesselClass, VesselRecord, VoyageRecord,
    VoyageStatus,
};
pub(crate) use super::semantic_key::{EntityKey, EntityKind};

pub(crate) fn build(scale: SupplyChainScale) -> BTreeMap<EntityKey, EntityRecord> {
    let mut entities = BTreeMap::new();
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

pub(crate) fn add_ports(scale: SupplyChainScale, entities: &mut BTreeMap<EntityKey, EntityRecord>) {
    for ordinal in 0..scale.ports {
        let region = match scale.region_index(ordinal) {
            0 => Region::NorthReach,
            1 => Region::SouthReach,
            index => Region::Generated(index as u16),
        };
        let name = match ordinal {
            0 => "Meridian".to_owned(),
            1 => "Southpoint".to_owned(),
            _ => format!("port-{:04x}-{ordinal}", scale.seeded(ordinal) as u16),
        };
        entities.insert(
            EntityKey::new(EntityKind::Port, ordinal as u32),
            EntityRecord::Port(PortRecord {
                code: 1000
                    + ordinal as u16
                    + if ordinal > 1 {
                        (scale.seeded(ordinal) % 7) as u16
                    } else {
                        0
                    },
                name,
                region,
                posture: OperatingPosture::Open,
            }),
        );
    }
}

pub(crate) fn add_terminals(
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
            EntityKey::new(EntityKind::Terminal, ordinal as u32),
            EntityRecord::Terminal(TerminalRecord {
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
                posture: OperatingPosture::Open,
            }),
        );
    }
}

pub(crate) fn add_berths(
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
            EntityKey::new(EntityKind::Berth, ordinal as u32),
            EntityRecord::Berth(BerthRecord {
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
                posture: OperatingPosture::Open,
            }),
        );
    }
}

pub(crate) fn add_vessels(
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
            EntityKey::new(EntityKind::Vessel, ordinal as u32),
            EntityRecord::Vessel(VesselRecord {
                call_sign,
                class: if ordinal == 2 {
                    VesselClass::HeavyLift
                } else if ordinal % 3 == 0 {
                    VesselClass::Panamax
                } else {
                    VesselClass::Feeder
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
                posture: OperatingPosture::Open,
            }),
        );
    }
}

pub(crate) fn add_voyages(
    scale: SupplyChainScale,
    entities: &mut BTreeMap<EntityKey, EntityRecord>,
) {
    for ordinal in 0..scale.voyages {
        entities.insert(
            EntityKey::new(EntityKind::Voyage, ordinal as u32),
            EntityRecord::Voyage(VoyageRecord {
                status: if ordinal == 1 {
                    VoyageStatus::Held
                } else {
                    VoyageStatus::Planned
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

pub(crate) fn add_port_calls(
    scale: SupplyChainScale,
    entities: &mut BTreeMap<EntityKey, EntityRecord>,
) {
    for ordinal in 0..scale.port_calls {
        entities.insert(
            EntityKey::new(EntityKind::PortCall, ordinal as u32),
            EntityRecord::PortCall(PortCallRecord {
                sequence: (ordinal % 3) as u16,
                revision: 0,
            }),
        );
    }
}

pub(crate) fn add_cargo(scale: SupplyChainScale, entities: &mut BTreeMap<EntityKey, EntityRecord>) {
    for ordinal in 0..scale.cargo_lots {
        let (hazard, booking) = match ordinal {
            0 => (HazardClass::Medical, BookingStatus::Booked),
            1 => (HazardClass::Industrial, BookingStatus::Booked),
            _ => (HazardClass::General, BookingStatus::Available),
        };
        entities.insert(
            EntityKey::new(EntityKind::CargoLot, ordinal as u32),
            EntityRecord::CargoLot(CargoLotRecord {
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

pub(crate) fn add_inspections(
    scale: SupplyChainScale,
    entities: &mut BTreeMap<EntityKey, EntityRecord>,
) {
    for ordinal in 0..scale.vessels {
        entities.insert(
            EntityKey::new(EntityKind::Inspection, ordinal as u32),
            EntityRecord::Inspection(InspectionRecord {
                result: if ordinal == 0 {
                    InspectionResult::Passed
                } else {
                    InspectionResult::Pending
                },
                minute: Minute(150 + ordinal as u32),
            }),
        );
    }
}
