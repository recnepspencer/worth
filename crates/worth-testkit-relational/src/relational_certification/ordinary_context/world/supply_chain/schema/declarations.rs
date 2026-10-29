pub use {std::collections::BTreeMap};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Region {
    NorthReach,
    SouthReach,
    Generated(u16),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum OperatingPosture {
    Open,
    Maintenance,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VesselClass {
    Feeder,
    Panamax,
    HeavyLift,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VoyageStatus {
    Planned,
    Delayed,
    Rerouted,
    Held,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HazardClass {
    General,
    Medical,
    Industrial,
    HazardousV2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BookingStatus {
    Available,
    Booked,
    Held,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum InspectionResult {
    Pending,
    Passed,
    Flagged,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Minute(pub u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Capacity(pub u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MassTons(pub u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DepthMeters(pub u16);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CargoCode(pub String);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HazardSchema {
    V1,
    V2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SchemaVersion {
    pub major: u16,
    pub minor: u16,
    pub hazard: HazardSchema,
}

impl SchemaVersion {
    pub const V1: Self = Self {
        major: 1,
        minor: 0,
        hazard: crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardSchema::V1,
    };

    pub const V2: Self = Self {
        major: 2,
        minor: 0,
        hazard: crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardSchema::V2,
    };
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PortRecord {
    pub code: u16,
    pub name: String,
    pub region: Region,
    pub posture: OperatingPosture,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TerminalRecord {
    pub name: String,
    pub capacity: Capacity,
    pub posture: OperatingPosture,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BerthRecord {
    pub name: String,
    pub depth: DepthMeters,
    pub capacity: Capacity,
    pub posture: OperatingPosture,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VesselRecord {
    pub call_sign: String,
    pub class: VesselClass,
    pub capacity: Capacity,
    pub posture: OperatingPosture,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VoyageRecord {
    pub status: VoyageStatus,
    pub departure: Minute,
    pub arrival: Minute,
    pub revision: u16,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PortCallRecord {
    pub sequence: u16,
    pub revision: u16,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CargoLotRecord {
    pub mass: MassTons,
    pub customer_code: CargoCode,
    pub hazard: HazardClass,
    pub booking: BookingStatus,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct InspectionRecord {
    pub result: InspectionResult,
    pub minute: Minute,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EntityRecord {
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
    pub fn kind(&self) -> EntityKind {
        match self {
            Self::Port(_) => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
            Self::Terminal(_) => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal,
            Self::Berth(_) => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth,
            Self::Vessel(_) => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel,
            Self::Voyage(_) => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage,
            Self::PortCall(_) => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
            Self::CargoLot(_) => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot,
            Self::Inspection(_) => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RelationContract {
    pub kind: RelationKind,
    pub source: EntityKind,
    pub target: EntityKind,
    pub min_per_source: u16,
    pub max_per_source: Option<u16>,
    pub symmetric: bool,
    pub ordered: bool,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RelationEdge {
    pub key: RelationKey,
    pub source: EntityKey,
    pub target: EntityKey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchemaError {
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
pub struct SupplyChainSchema {
    pub version: SchemaVersion,
    pub relations: BTreeMap<RelationKind, RelationContract>,
}

impl SupplyChainSchema {
    pub fn canonical(version: SchemaVersion) -> Self {
        let contracts = [
            (
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                1,
                Some(1),
                false,
                false,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::BerthAtTerminal,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal,
                1,
                Some(1),
                false,
                false,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth,
                0,
                Some(1),
                false,
                false,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageUsesVessel,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel,
                1,
                Some(1),
                false,
                false,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
                2,
                None,
                false,
                true,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                1,
                Some(1),
                false,
                false,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
                0,
                None,
                false,
                true,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage,
                0,
                Some(1),
                false,
                false,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::InspectionCoversVessel,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel,
                1,
                Some(1),
                false,
                false,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
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