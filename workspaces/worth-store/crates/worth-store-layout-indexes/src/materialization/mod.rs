mod admission;
mod coverage;
mod coverage_basis;
mod denial;
mod freshness;
#[cfg(test)]
mod runtime_tests;
mod source;
#[cfg(test)]
mod source_binding_tests;
mod state;
#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
pub(crate) mod tests;
mod watermark;

pub use admission::{
    btree_publication_materialization_admission_cases,
    catalog_root_materialization_admission_cases,
    imported_blob_materialization_admission_cases, lsm_lookup_materialization_admission_cases,
    lsm_publication_materialization_admission_cases, lsm_replay_materialization_admission_cases,
    AdmittedLayoutMaterialization,
    BTreePublicationMaterializationAdmissionCaseId,
    BTreePublicationMaterializationAdmissionOutcome, BTreePublicationMaterializationAdmissionView,
    CatalogRootMaterializationAdmissionCaseId,
    CatalogRootMaterializationAdmissionOutcome, CatalogRootMaterializationAdmissionView,
    ImportedBlobMaterializationAdmissionCaseId, ImportedBlobMaterializationAdmissionOutcome,
    ImportedBlobMaterializationAdmissionView, LsmLookupMaterializationAdmissionCaseId,
    LsmLookupMaterializationAdmissionOutcome, LsmLookupMaterializationAdmissionView,
    LsmPublicationMaterializationAdmissionCaseId, LsmPublicationMaterializationAdmissionOutcome,
    LsmPublicationMaterializationAdmissionView, LsmReplayMaterializationAdmissionCaseId,
    LsmReplayMaterializationAdmissionOutcome, LsmReplayMaterializationAdmissionView,
};
pub(crate) use coverage::LayoutCoverageWitness;
pub use coverage_basis::AdmittedCoverageBasis;
#[cfg(test)]
pub use denial::CoverageGapWitness;
pub use denial::{MaterializationDenial, MaterializationDenialKind};
pub use freshness::{
    CurrentLayoutMaterialization, CurrentMaterializationFrontier, MaterializationFreshness,
    StaleLayoutMaterialization,
};
pub use source::{
    ImportedBlobMaterializationSourceIdentity, LayoutMaterializationSourceIdentity,
    LayoutMaterializationSourceKind,
};
pub(crate) use state::{LayoutMaterializationState, MaterializationStateClass};
pub(crate) use watermark::{CoverageBasisKind, LayoutWatermark, PhysicalCoverageBasis};
