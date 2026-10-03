#[cfg(test)]
mod bridge_snapshot_reader_tests;
#[cfg(test)]
mod bridge_source_tests;
mod change_publication;
#[cfg(test)]
mod consistency_lowering_tests;
mod identities;
#[cfg(test)]
mod identity_boundary_tests;
pub(crate) mod identity_parts;
mod lowering_precision;
pub(crate) mod patch_envelopes;
#[cfg(test)]
mod patch_envelopes_tests;
mod publication_outcome;
#[cfg(test)]
mod relational_test_support;
mod runtime_source;
#[cfg(test)]
mod snapshot_catalog_tests;
mod snapshot_reading;
#[cfg(test)]
mod snapshot_reading_tests;
mod snapshot_values;
#[cfg(test)]
mod test_catalog;

pub use change_publication::{
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
    PendingRelationalBridgeCanonicalSubscription, RelationalBridgeBranchHeadLease,
    RelationalBridgeBranchHeadReleaseReceipt, RelationalBridgeCanonicalSubscription,
    RelationalBridgeObservationLease, RelationalBridgeObservationReleaseReceipt,
    RelationalBridgeRetainedSnapshot, RelationalBridgeSourceConfigurationError,
    RuntimeBridgeRelationalSource,
};
#[cfg(test)]
pub use test_catalog::{PublicationBridgeCatalog, PublicationBridgeSnapshot};
