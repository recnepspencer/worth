//! The Bridge's Relational side: the adapter that reads a Relational runtime
//! through its change source, the identity parts the Bridge carries for
//! Relational records.

pub use crate::relational_source::identity_parts::{
    RelationalBridgeRecordIdentityKind, RelationalBridgeRecordIdentityParts,
    RelationalBridgeSnapshotIdentityParts,
};
pub use crate::relational_source::{
    bridge_snapshot_identity_for_commit, bridge_snapshot_identity_for_handle,
    PendingRelationalBridgeCanonicalSubscription, RelationalBridgeBasisRetirementDenial,
    RelationalBridgeBasisRetirementProgress, RelationalBridgeBranchHeadLease,
    RelationalBridgeBranchHeadReleaseReceipt, RelationalBridgeBranchHeadRetirement,
    RelationalBridgeCanonicalSubscription, RelationalBridgeObservationLease,
    RelationalBridgeObservationReleaseReceipt, RelationalBridgeObservationRetirement,
    RelationalBridgePatchPublication, RelationalBridgePublicationDeferred,
    RelationalBridgePublicationDenial, RelationalBridgePublicationFailure,
    RelationalBridgePublicationOutcome, RelationalBridgePublicationRebindRequired,
    RelationalBridgePublicationStale, RelationalBridgeRetainedSnapshot,
    RelationalBridgeRetainedSnapshotRetirement, RelationalBridgeRetainedSnapshotRetirementDenial,
    RelationalBridgeRetainedSnapshotRetirementProgress, RelationalBridgeSourceConfigurationError,
    RelationalOpaqueAspectWideningAdmission, RelationalOpaqueAspectWideningAdmissionDenial,
    RetiredRelationalBridgeBranchHead, RetiredRelationalBridgeCanonicalSubscription,
    RetiredRelationalBridgeObservation, RetiredRelationalBridgeRetainedSnapshot,
    RetiredRuntimeBridgeRelationalSource, RuntimeBridgeRelationalSource,
};
