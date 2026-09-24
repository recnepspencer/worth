mod authority;
mod classification;
mod derived;

// No resident owner issues dispositions yet; the adapters stay proven by the
// unit tests until the first C.11 owner consumer wires them.
#[cfg(test)]
use authority::{
    project_resident_current_root_selector_authority,
    project_resident_previous_root_selector_authority, project_resident_root_manifest_authority,
    StoreOwnerDispositionAdapterDenial,
};
pub use authority::{DamagedPhysicalAuthorityObservation, IntactPhysicalAuthorityObservation};
pub use classification::{
    OwnerDispositionProjectionDenial, PhysicalArtifactDisposition, PhysicalArtifactRoleDisposition,
};
pub use derived::{
    DamagedPhysicalDerivedDisposition, IndeterminateDerivedRebuildability,
    IntactPhysicalDerivedObservation, RebuildablePhysicalDerivedObservation,
    UnknownDerivedRebuildability,
};

#[cfg(test)]
mod tests;
