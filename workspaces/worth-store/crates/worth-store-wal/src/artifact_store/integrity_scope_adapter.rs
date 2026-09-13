use worth_store_physical_format::WalSegmentIdentity;

use super::WalSegmentArtifactIdentity;

/// Maps Store-owned WAL artifact identity into the canonical pre-validation
/// scope identity. LSN fields are intentionally unavailable at this boundary.
pub const fn wal_frame_integrity_scope_identity(
    artifact: WalSegmentArtifactIdentity,
) -> WalSegmentIdentity {
    artifact.format_identity()
}
