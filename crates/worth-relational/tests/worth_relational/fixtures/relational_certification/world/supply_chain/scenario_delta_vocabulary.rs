pub(crate) use super::schema::SchemaVersion;
pub(crate) use super::semantic_key::{BranchLabel, EntityKey, RelationKey};

/// The closed semantic delta vocabulary shared by contracts, reads, and the
/// pure oracle.  Its meaning does not depend on how a delta is interpreted.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum DeltaId {
    StormRerouteAurora,
    MaintainAtlasBerth,
    HoldMedicalCargo,
    ExpandSouthpointCapacity,
    CompetingAuroraArrival,
    RetireAtlasWhileInspectingAurora,
    RewireAuroraPortCall,
    AdoptHazardClassificationV2,
}

pub(crate) type SupplyChainScenarioDelta = DeltaId;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum DeltaPrecondition {
    EntityPresent(EntityKey),
    RelationPresent(RelationKey),
    Schema(SchemaVersion),
    Branch(BranchLabel),
    DeltaNotAccepted(DeltaId),
}
