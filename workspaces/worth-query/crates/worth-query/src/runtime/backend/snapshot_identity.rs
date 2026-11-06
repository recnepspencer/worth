use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;

pub(in crate::runtime) fn unavailable_snapshot_identity() -> WorthQuerySnapshotIdentity {
    // This private descriptor has four fixed literal entries and no caller
    // material; it fits both standard digest limits by construction.
    WorthQuerySnapshotIdentity::preview(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::RuntimeStateSnapshot)
            .field_shape(
                WorthQueryEvidenceTag::new("snapshot_authority"),
                "unavailable",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("snapshot_contract"),
                "backend-must-override-for-authoritative-truth",
            )
            .seal()
            .expect("the fixed unavailable-snapshot descriptor fits the standard digest policy"),
    )
}
