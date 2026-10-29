pub(crate) use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum EntityKind {
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
pub(crate) struct EntityKey {
    pub(crate) kind: EntityKind,
    pub(crate) ordinal: u32,
}

impl EntityKey {
    pub(crate) const fn new(kind: EntityKind, ordinal: u32) -> Self {
        Self { kind, ordinal }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum RelationKind {
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
pub(crate) struct RelationKey {
    pub(crate) kind: RelationKind,
    pub(crate) ordinal: u32,
}

impl RelationKey {
    pub(crate) const fn new(kind: RelationKind, ordinal: u32) -> Self {
        Self { kind, ordinal }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum FieldKey {
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
    pub(crate) const ALL: [Self; 20] = [
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
pub(crate) enum BranchLabel {
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
pub(crate) enum AbsenceKind {
    Entity,
    Relation,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct SemanticPath {
    pub(crate) entity: EntityKey,
    pub(crate) field: Option<FieldKey>,
}

impl SemanticPath {
    pub(crate) const fn entity(entity: EntityKey) -> Self {
        Self {
            entity,
            field: None,
        }
    }

    pub(crate) const fn field(entity: EntityKey, field: FieldKey) -> Self {
        Self {
            entity,
            field: Some(field),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Anchor {
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
    pub(crate) const fn entity(self) -> EntityKey {
        match self {
            Self::Meridian => EntityKey::new(EntityKind::Port, 0),
            Self::Southpoint => EntityKey::new(EntityKind::Port, 1),
            Self::MeridianContainer => EntityKey::new(EntityKind::Terminal, 0),
            Self::SouthpointContainer => EntityKey::new(EntityKind::Terminal, 1),
            Self::Atlas => EntityKey::new(EntityKind::Berth, 0),
            Self::Beacon => EntityKey::new(EntityKind::Berth, 1),
            Self::SouthpointBerth => EntityKey::new(EntityKind::Berth, 2),
            Self::Aurora => EntityKey::new(EntityKind::Vessel, 0),
            Self::AuroraEastbound => EntityKey::new(EntityKind::Voyage, 0),
            Self::AuroraMeridian => EntityKey::new(EntityKind::PortCall, 0),
            Self::AuroraSouthpoint => EntityKey::new(EntityKind::PortCall, 1),
            Self::MedicalSupplies => EntityKey::new(EntityKind::CargoLot, 0),
            Self::MachineParts => EntityKey::new(EntityKind::CargoLot, 1),
            Self::AuroraArrival => EntityKey::new(EntityKind::Inspection, 0),
        }
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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
