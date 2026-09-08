mod diagnostics;
mod disposition;
#[cfg(feature = "recovery-runtime-owner")]
mod recovery_wal_admission;
#[allow(
    dead_code,
    reason = "Wave A establishes family cutover seams before record-serving consumers move"
)]
pub(in crate::physical_runtime) mod resident_admission;
mod root_protocol_admission_denial;
mod scrub;

pub use diagnostics::{
    PhysicalRootProtocolRoute, ResidentAdmissionCounters, RootProtocolRouteCounters,
};
pub(in crate::physical_runtime) use diagnostics::{
    ResidentAdmissionCounterCells, RootProtocolRouteCounterCells,
};
pub(in crate::physical_runtime) use disposition::{
    project_resident_current_root_selector_authority,
    project_resident_previous_root_selector_authority, project_resident_root_manifest_authority,
    StoreOwnerDispositionAdapterDenial,
};
pub use disposition::{
    DamagedPhysicalAuthorityObservation, DamagedPhysicalDerivedDisposition,
    IndeterminateDerivedRebuildability, IntactPhysicalAuthorityObservation,
    IntactPhysicalDerivedObservation, OwnerDispositionProjectionDenial,
    PhysicalArtifactDisposition, PhysicalArtifactRoleDisposition,
    RebuildablePhysicalDerivedObservation, UnknownDerivedRebuildability,
};
#[cfg(feature = "recovery-runtime-owner")]
pub use recovery_wal_admission::{
    IntegrityAdmittedRecoveryWalFrame, IntegrityAdmittedRecoveryWalSegment,
    RecoveryWalIntegrityAdmissionDenial,
};
pub(in crate::physical_runtime) use resident_admission::denial::ResidentIntegrityAdmissionDenial;
pub(in crate::physical_runtime) use resident_admission::extent::{
    admit_resident_extent_chunk, admit_resident_extent_manifest,
};
pub(in crate::physical_runtime) use resident_admission::load::ResidentAdmissionContext;
pub(in crate::physical_runtime) use resident_admission::page::{
    admit_resident_page, IntegrityAdmittedResidentPageBasis,
};
pub use root_protocol_admission_denial::RootProtocolAdmissionDenial;
pub(in crate::physical_runtime) use scrub::PhysicalIntegrityScrubOwner;
pub use scrub::{
    ManagedPhysicalIntegrityScrubHandle, ManagedPhysicalIntegrityScrubProgress,
    ManagedPhysicalIntegrityScrubRequest, PhysicalIntegrityRuntimeReportContext,
    PhysicalIntegrityRuntimeReportDenial, PhysicalIntegrityScrubCancellation,
    PhysicalIntegrityScrubCounters, PhysicalIntegrityScrubDeferral,
    PhysicalIntegrityScrubRequestDenial, PhysicalIntegrityScrubResume,
    PhysicalIntegrityScrubTarget, PhysicalIntegrityScrubWindowObservation,
};
