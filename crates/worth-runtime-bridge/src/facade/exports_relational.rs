//! The Bridge's Relational side: the adapter that reads a Relational runtime
//! through its change source, the identity parts the Bridge carries for
//! Relational records, and grouped truth over Relational snapshot reads.

pub use crate::relational_grouped_truth::{
    encode_snapshot_aspect_read_value, materialize_relational_authoritative_row_set,
    project_relational_grouped_truth, RelationalAuthoritativeRowArtifact,
    RelationalAuthoritativeRowSetArtifact, RelationalGroupedMemberRow,
    RelationalGroupedProjectionArtifact, RelationalGroupedProjectionContract,
    RelationalGroupedProjectionDigest, RelationalGroupedTruthError,
    RelationalProjectedAspectValueSet, RelationalRowIdentity, RelationalRowSetDigest,
};
pub use crate::relational_source::identity_parts::{
    RelationalBridgeRecordIdentityKind, RelationalBridgeRecordIdentityParts,
    RelationalBridgeSnapshotIdentityParts,
};
pub use crate::relational_source::{
    bridge_snapshot_identity_for_commit, bridge_snapshot_identity_for_handle,
    RelationalBridgeBranchHeadLease, RelationalBridgeBranchHeadReleaseReceipt,
    RelationalBridgeObservationLease, RelationalBridgeObservationReleaseReceipt,
    RelationalBridgePatchPublication, RelationalBridgePublicationDeferred,
    RelationalBridgePublicationDenial, RelationalBridgePublicationFailure,
    RelationalBridgePublicationOutcome, RelationalBridgePublicationRebindRequired,
    RelationalBridgePublicationStale, RelationalBridgeRetainedSnapshot,
    RelationalBridgeSourceConfigurationError, RelationalOpaqueAspectWideningAdmission,
    RelationalOpaqueAspectWideningAdmissionDenial, RuntimeBridgeRelationalSource,
};
