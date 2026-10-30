pub use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EntityKind {
    Port,
    Terminal,
    Berth,
    Vessel,
    Voyage,
    PortCall,
    CargoLot,
    Inspection,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntityKey {
    pub kind: EntityKind,
    pub ordinal: u32,
}

impl EntityKey {
    pub const fn new(kind: EntityKind, ordinal: u32) -> Self {
        Self { kind, ordinal }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RelationKind {
    TerminalAtPort,
    BerthAtTerminal,
    VesselAssignedToBerth,
    VoyageUsesVessel,
    VoyageHasCall,
    CallAtPort,
    CallPrecedes,
    CargoBookedOnVoyage,
    InspectionCoversVessel,
    SharesPilotageZone,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RelationKey {
    pub kind: RelationKind,
    pub ordinal: u32,
}

impl RelationKey {
    pub const fn new(kind: RelationKind, ordinal: u32) -> Self {
        Self { kind, ordinal }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FieldKey {
    Name,
    PortCode,
    CallSign,
    Region,
    Posture,
    Capacity,
    Depth,
    Class,
    Status,
    DepartureMinute,
    ArrivalMinute,
    Revision,
    Sequence,
    Mass,
    CustomerCode,
    HazardClass,
    BookingStatus,
    InspectionResult,
    InspectionMinute,
    SchemaMeaning,
}

impl FieldKey {
    pub const ALL: [Self; 20] = [
        Self::Name,
        Self::PortCode,
        Self::CallSign,
        Self::Region,
        Self::Posture,
        Self::Capacity,
        Self::Depth,
        Self::Class,
        Self::Status,
        Self::DepartureMinute,
        Self::ArrivalMinute,
        Self::Revision,
        Self::Sequence,
        Self::Mass,
        Self::CustomerCode,
        Self::HazardClass,
        Self::BookingStatus,
        Self::InspectionResult,
        Self::InspectionMinute,
        Self::SchemaMeaning,
    ];
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BranchLabel {
    Operating,
    Storm,
    Maintenance,
    Customs,
    MedicalHold,
    SouthpointExpansion,
    CompetingArrival,
    Inspection,
    Rewire,
    HazardV2,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AbsenceKind {
    Entity,
    Relation,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticPath {
    pub entity: EntityKey,
    pub field: Option<FieldKey>,
}

impl SemanticPath {
    pub const fn entity(entity: EntityKey) -> Self {
        Self {
            entity,
            field: None,
        }
    }

    pub const fn field(entity: EntityKey, field: FieldKey) -> Self {
        Self {
            entity,
            field: Some(field),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Anchor {
    Meridian,
    Southpoint,
    MeridianContainer,
    SouthpointContainer,
    Atlas,
    Beacon,
    SouthpointBerth,
    Aurora,
    AuroraEastbound,
    AuroraMeridian,
    AuroraSouthpoint,
    MedicalSupplies,
    MachineParts,
    AuroraArrival,
}

impl Anchor {
    pub const fn entity(self) -> EntityKey {
        match self {
            Self::Meridian => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 0),
            Self::Southpoint => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 1),
            Self::MeridianContainer => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal, 0),
            Self::SouthpointContainer => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal, 1),
            Self::Atlas => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 0),
            Self::Beacon => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 1),
            Self::SouthpointBerth => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 2),
            Self::Aurora => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel, 0),
            Self::AuroraEastbound => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 0),
            Self::AuroraMeridian => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, 0),
            Self::AuroraSouthpoint => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, 1),
            Self::MedicalSupplies => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, 0),
            Self::MachineParts => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, 1),
            Self::AuroraArrival => crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection, 0),
        }
    }
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Meridian => "world.ports.meridian",
            Self::Southpoint => "world.ports.southpoint",
            Self::MeridianContainer => "world.terminals.meridian_container",
            Self::SouthpointContainer => "world.terminals.southpoint_container",
            Self::Atlas => "world.berths.atlas",
            Self::Beacon => "world.berths.beacon",
            Self::SouthpointBerth => "world.berths.southpoint",
            Self::Aurora => "world.vessels.aurora",
            Self::AuroraEastbound => "world.voyages.aurora_eastbound",
            Self::AuroraMeridian => "world.calls.aurora_meridian",
            Self::AuroraSouthpoint => "world.calls.aurora_southpoint",
            Self::MedicalSupplies => "world.cargo.medical_supplies",
            Self::MachineParts => "world.cargo.machine_parts",
            Self::AuroraArrival => "world.inspections.aurora_arrival",
        };
        formatter.write_str(value)
    }
}
