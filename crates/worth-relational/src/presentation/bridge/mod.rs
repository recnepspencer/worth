mod authoritative_patch_publication;
mod authoritative_publication_witness;
#[cfg(test)]
mod bridge_snapshot_reader_tests;
#[cfg(test)]
mod bridge_source_tests;
mod identities;
mod partition_projection;
#[cfg(test)]
mod patch_binding_authority_tests;
pub(crate) mod patch_envelopes;
#[cfg(test)]
mod patch_envelopes_tests;
mod patch_semantic_validation;
mod publication_outcome;
mod runtime_source;
#[cfg(test)]
mod snapshot_catalog_tests;
mod snapshot_reading;
#[cfg(test)]
mod snapshot_reading_tests;
mod snapshot_values;
#[cfg(test)]
mod test_catalog;

pub use authoritative_patch_publication::{
    RelationalOpaqueAspectWideningAdmission, RelationalOpaqueAspectWideningAdmissionDenial,
};
pub use identities::{bridge_snapshot_identity_for_commit, bridge_snapshot_identity_for_handle};
pub use publication_outcome::{
    RelationalBridgePatchPublication, RelationalBridgePublicationDeferred,
    RelationalBridgePublicationDenial, RelationalBridgePublicationFailure,
    RelationalBridgePublicationOutcome, RelationalBridgePublicationRebindRequired,
    RelationalBridgePublicationStale,
};
pub use runtime_source::{
    RelationalBridgeBranchHeadLease, RelationalBridgeBranchHeadReleaseReceipt,
    RelationalBridgeObservationLease, RelationalBridgeObservationReleaseReceipt,
    RelationalBridgeRetainedSnapshot, RelationalBridgeSourceConfigurationError,
    RuntimeBridgeRelationalSource,
};
#[cfg(test)]
pub use test_catalog::{PublicationBridgeCatalog, PublicationBridgeSnapshot};
