mod artifact_name;
mod segment_inspection;

pub(super) use artifact_name::wal_segment_relative_path;
pub use artifact_name::WalSegmentArtifactIdentity;
pub use segment_inspection::{
    inspect_bounded_wal_active_tail_with_evidence, inspect_complete_wal_segment,
    inspect_interrupted_wal_segment_start, inspect_verified_wal_active_tail,
    inspect_verified_wal_segment, InterruptedWalSegmentStart, InterruptedWalTail,
    VerifiedWalActiveTail, VerifiedWalArtifact, VerifiedWalFrame, VerifiedWalFramePayload,
    VerifiedWalSegment, WalActiveTailInspectionDenial, WalActiveTailInspectionFailure,
    WalSegmentInspection,
};

#[cfg(test)]
mod tests;
