//! Store contract vocabulary.
//!
//! External callers cannot mint physical authority witnesses directly:
//!
//! ```compile_fail
//! use worth_store_contracts::{
//!     PhysicalAuthorityScope, StorePhysicalAuthorityWitness, ROADMAP_2_S1_SCOPE,
//! };
//!
//! let _forged = StorePhysicalAuthorityWitness {
//!     roadmap_scope: ROADMAP_2_S1_SCOPE,
//!     authority_scope: PhysicalAuthorityScope::PhysicalEvidenceExport,
//! };
//! ```

#![forbid(unsafe_code)]

mod artifact_identity;
mod contract_error;
mod corruption_handoff_damage_case;
mod durable_artifact_family_classification;
mod durable_artifact_family_id;
mod existing_artifact_family;
mod handoff_readiness;
mod physical_authority;
mod physical_integrity;
mod resources;
mod roadmap_scope;

pub use artifact_identity::{StableArtifactId, StableDigest};
pub use contract_error::{StoreContractError, StoreContractResult};
pub use corruption_handoff_damage_case::CorruptionHandoffDamageCase;
pub use durable_artifact_family_classification::{
    ArtifactFamilyAccessLane, ArtifactFamilyAuthorityClass, ArtifactFamilyLifecycleClass,
    DurableArtifactMigrationPosture, DurableArtifactOwningBoundary, DurableArtifactProjectionClass,
    DurableArtifactRebuildPosture,
};
pub use durable_artifact_family_id::DurableArtifactFamilyId;
pub use existing_artifact_family::{
    DerivedFamilyRetentionPolicy, LayoutCompactionFamilyKind, LayoutFamilyCompactionUnit,
    MaintenanceArtifactFamily, PlacementArtifactFamily, PublicationFamily, SupportArtifactFamily,
    WalRecordFamily,
};
pub use handoff_readiness::{
    AcceptedHandoffReadiness, HandoffEvidenceDigestSet, S0HandoffArtifactKind,
};
pub use physical_authority::{
    PhysicalAuthorityBoundaryInstance, PhysicalAuthorityScope, StorePhysicalAuthorityWitness,
    ROADMAP_2_PRIMARY_PHYSICAL_BOUNDARY, ROADMAP_2_REPLAY_PHYSICAL_BOUNDARY,
};
pub use physical_integrity::readiness::{
    BoundedCounterRecap, BufferPoolAuthorityRecap, DenialBehaviorRecap, DeniedBoundaryKind,
    PhysicalAuthorityRecap,
};
pub use physical_integrity::readiness::{
    IntegrityInspectionLifetimeLaw, NoMaterializationWitness, PhysicalIntegrityReadinessPayload,
    ProtectedIntegrityViewCapability, ScrubPlanningAllocationEnvelope, VerifierResidentEnvelope,
};
pub use physical_integrity::readiness::{
    PhysicalIntegrityReadinessDenial, PhysicalIntegrityReadinessDenialKind,
};
pub use resources::{
    BackgroundPressureDeclaration, BackgroundPressureKind, QueueProducerKind,
    QueueProducerResourceShape,
};
pub use roadmap_scope::{
    RoadmapScope, ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE, ROADMAP_2_S1_SCOPE, ROADMAP_2_SCOPE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableArtifactClass {
    Authoritative,
    DerivedDurable,
    Ephemeral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedAccuracyClass {
    Exact,
    Conservative,
    Approximate,
    Heuristic,
    Advisory,
}
