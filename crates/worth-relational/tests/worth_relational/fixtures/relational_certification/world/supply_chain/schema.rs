pub(crate) use std::collections::BTreeMap;

pub(crate) use super::semantic_key::{EntityKey, EntityKind, RelationKey, RelationKind};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Region {
    NorthReach,
    SouthReach,
    Generated(u16),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum OperatingPosture {
    Open,
    Maintenance,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum VesselClass {
    Feeder,
    Panamax,
    HeavyLift,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum VoyageStatus {
    Planned,
    Delayed,
    Rerouted,
    Held,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum HazardClass {
    General,
    Medical,
    Industrial,
    HazardousV2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum BookingStatus {
    Available,
    Booked,
    Held,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum InspectionResult {
    Pending,
    Passed,
    Flagged,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Minute(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Capacity(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct MassTons(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct DepthMeters(pub(crate) u16);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CargoCode(pub(crate) String);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum HazardSchema {
    V1,
    V2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SchemaVersion {
    pub(crate) major: u16,
    pub(crate) minor: u16,
    pub(crate) hazard: HazardSchema,
}

impl SchemaVersion {
    pub(crate) const V1: Self = Self {
        major: 1,
        minor: 0,
        hazard: HazardSchema::V1,
    };

    pub(crate) const V2: Self = Self {
        major: 2,
        minor: 0,
        hazard: HazardSchema::V2,
    };
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PortRecord {
    pub(crate) code: u16,
    pub(crate) name: String,
    pub(crate) region: Region,
    pub(crate) posture: OperatingPosture,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct TerminalRecord {
    pub(crate) name: String,
    pub(crate) capacity: Capacity,
    pub(crate) posture: OperatingPosture,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct BerthRecord {
    pub(crate) name: String,
    pub(crate) depth: DepthMeters,
    pub(crate) capacity: Capacity,
    pub(crate) posture: OperatingPosture,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct VesselRecord {
    pub(crate) call_sign: String,
    pub(crate) class: VesselClass,
    pub(crate) capacity: Capacity,
    pub(crate) posture: OperatingPosture,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct VoyageRecord {
    pub(crate) status: VoyageStatus,
    pub(crate) departure: Minute,
    pub(crate) arrival: Minute,
    pub(crate) revision: u16,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PortCallRecord {
    pub(crate) sequence: u16,
    pub(crate) revision: u16,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CargoLotRecord {
    pub(crate) mass: MassTons,
    pub(crate) customer_code: CargoCode,
    pub(crate) hazard: HazardClass,
    pub(crate) booking: BookingStatus,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct InspectionRecord {
    pub(crate) result: InspectionResult,
    pub(crate) minute: Minute,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum EntityRecord {
    Port(PortRecord),
    Terminal(TerminalRecord),
    Berth(BerthRecord),
    Vessel(VesselRecord),
    Voyage(VoyageRecord),
    PortCall(PortCallRecord),
    CargoLot(CargoLotRecord),
    Inspection(InspectionRecord),
}

impl EntityRecord {
    pub(crate) fn kind(&self) -> EntityKind {
        match self {
            Self::Port(_) => EntityKind::Port,
            Self::Terminal(_) => EntityKind::Terminal,
            Self::Berth(_) => EntityKind::Berth,
            Self::Vessel(_) => EntityKind::Vessel,
            Self::Voyage(_) => EntityKind::Voyage,
            Self::PortCall(_) => EntityKind::PortCall,
            Self::CargoLot(_) => EntityKind::CargoLot,
            Self::Inspection(_) => EntityKind::Inspection,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct RelationContract {
    pub(crate) kind: RelationKind,
    pub(crate) source: EntityKind,
    pub(crate) target: EntityKind,
    pub(crate) min_per_source: u16,
    pub(crate) max_per_source: Option<u16>,
    pub(crate) symmetric: bool,
    pub(crate) ordered: bool,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct RelationEdge {
    pub(crate) key: RelationKey,
    pub(crate) source: EntityKey,
    pub(crate) target: EntityKey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SchemaError {
    UnknownRelation(RelationKind),
    InvalidEndpoint {
        relation: RelationKind,
        source: EntityKind,
        target: EntityKind,
    },
    DuplicateRelation(RelationKey),
    MinimumCardinality(RelationKind, EntityKey),
    CardinalityExceeded(RelationKind, EntityKey),
    MissingSymmetricReverse(RelationKey),
    OrderedRouteViolation(RelationKey),
    RouteCycle,
    DuplicateVoyageCall(EntityKey),
    MissingRouteLink {
        voyage: EntityKey,
        source: EntityKey,
        target: EntityKey,
    },
    OrphanRouteLink(RelationKey),
    HazardMeaningViolation {
        entity: EntityKey,
        schema: SchemaVersion,
        hazard: HazardClass,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SupplyChainSchema {
    pub(crate) version: SchemaVersion,
    pub(crate) relations: BTreeMap<RelationKind, RelationContract>,
}

impl SupplyChainSchema {
    pub(crate) fn canonical(version: SchemaVersion) -> Self {
        let contracts = [
            (
                RelationKind::TerminalAtPort,
                EntityKind::Terminal,
                EntityKind::Port,
                1,
                Some(1),
                false,
                false,
            ),
            (
                RelationKind::BerthAtTerminal,
                EntityKind::Berth,
                EntityKind::Terminal,
                1,
                Some(1),
                false,
                false,
            ),
            (
                RelationKind::VesselAssignedToBerth,
                EntityKind::Vessel,
                EntityKind::Berth,
                0,
                Some(1),
                false,
                false,
            ),
            (
                RelationKind::VoyageUsesVessel,
                EntityKind::Voyage,
                EntityKind::Vessel,
                1,
                Some(1),
                false,
                false,
            ),
            (
                RelationKind::VoyageHasCall,
                EntityKind::Voyage,
                EntityKind::PortCall,
                2,
                None,
                false,
                true,
            ),
            (
                RelationKind::CallAtPort,
                EntityKind::PortCall,
                EntityKind::Port,
                1,
                Some(1),
                false,
                false,
            ),
            (
                RelationKind::CallPrecedes,
                EntityKind::PortCall,
                EntityKind::PortCall,
                0,
                None,
                false,
                true,
            ),
            (
                RelationKind::CargoBookedOnVoyage,
                EntityKind::CargoLot,
                EntityKind::Voyage,
                0,
                Some(1),
                false,
                false,
            ),
            (
                RelationKind::InspectionCoversVessel,
                EntityKind::Inspection,
                EntityKind::Vessel,
                1,
                Some(1),
                false,
                false,
            ),
            (
                RelationKind::SharesPilotageZone,
                EntityKind::Port,
                EntityKind::Port,
                0,
                None,
                true,
                false,
            ),
        ]
        .into_iter()
        .map(
            |(kind, source, target, min_per_source, max_per_source, symmetric, ordered)| {
                (
                    kind,
                    RelationContract {
                        kind,
                        source,
                        target,
                        min_per_source,
                        max_per_source,
                        symmetric,
                        ordered,
                    },
                )
            },
        )
        .collect();
        Self {
            version,
            relations: contracts,
        }
    }
}
