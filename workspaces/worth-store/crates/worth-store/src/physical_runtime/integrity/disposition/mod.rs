mod authority;
mod classification;
mod derived;

pub(in crate::physical_runtime) use authority::{
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
