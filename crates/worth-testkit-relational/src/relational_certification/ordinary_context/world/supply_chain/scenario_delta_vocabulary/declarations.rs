pub use crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion;
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey,
};

/// The closed semantic delta vocabulary shared by contracts, reads, and the
/// pure oracle.  Its meaning does not depend on how a delta is interpreted.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeltaId {
    StormRerouteAurora,
    MaintainAtlasBerth,
    HoldMedicalCargo,
    ExpandSouthpointCapacity,
    CompetingAuroraArrival,
    RetireAtlasWhileInspectingAurora,
    RewireAuroraPortCall,
    AdoptHazardClassificationV2,
}

pub type SupplyChainScenarioDelta = DeltaId;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DeltaPrecondition {
    EntityPresent(EntityKey),
    RelationPresent(RelationKey),
    Schema(SchemaVersion),
    Branch(BranchLabel),
    DeltaNotAccepted(DeltaId),
}
