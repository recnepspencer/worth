//! Roadmap 2 S.7 native content-addressed blob and object chunk store.
//!
//! # Legal blob lifecycle order
//!
//! Callers follow proof flow through facade-returned capabilities in this order:
//!
//! ```text
//! identity → integrity → dedupe → streaming → lifecycle → publication
//! → reachability → placement → compaction → corruption → recovery → retention/reclaim
//! ```
//!
//! Each lifecycle stage exports capabilities, outcomes, denials, and counter
//! witnesses in that authority order. Hostile-lane `reject_*` constructors are
//! grouped separately and must not be mixed with production admission handles.
//!
//! Construction-boundary compile-fail proofs live in [`compile_fail`] and the
//! `#[doc(hidden)]` compile-fail modules re-exported below.
#![forbid(unsafe_code)]

mod backup_verification;
mod capsule_readiness;
mod chunk_identity;
mod chunk_integrity;
#[cfg(feature = "certification-test-authority")]
mod closeout_bundle;
mod compaction;
mod compile_fail;
mod corruption;
mod dedupe;
mod export_bundle;
mod exports;
mod handoffs;
#[cfg(feature = "certification-test-authority")]
mod harness_execution;
#[cfg(feature = "certification-test-authority")]
mod heavy_fixture;
mod import_readmission;
mod layout_projection;
mod lifecycle;
mod placement;
mod publication;
mod reachability;
mod recovery;
mod retention_reclaim;
mod streaming;
#[cfg(any(test, feature = "certification-test-authority"))]
mod test_support;

pub use backup_verification::{
    verify_bounded_blob_backup_artifact, verify_bounded_blob_backup_artifact_from_reader,
    BlobBackupChunkArtifact, BoundedBlobBackupDenial, BoundedBlobBackupObservation,
    BoundedBlobBackupVerificationRequest,
};
#[cfg(feature = "certification-test-authority")]
pub use closeout_bundle::ExecutedBlobLifecycleEvidenceBundle;
#[cfg(all(test, feature = "certification-test-authority"))]
pub(crate) use compaction::test_support::{
    authority as phase25_compaction_authority,
    compacted_rewritten_publication as phase25_compacted_rewritten_publication,
    intent as phase25_compaction_intent,
    verified_read_for_rewritten as phase25_verified_read_for_rewritten,
};
#[cfg(test)]
pub use exports::hostile_lane::*;
pub use exports::*;
#[cfg(feature = "certification-test-authority")]
pub use heavy_fixture::*;
pub use layout_projection::{
    reject_chunk_tree_root_as_blob_object_layout_authority,
    reject_full_blob_buffer_as_streaming_layout_authority,
    reject_streaming_frontier_as_chunk_tree_layout_authority,
    BlobGenerationPublicationLayoutReport, BlobLayoutAccessDenial, BlobLayoutAccessDenialKind,
    BlobLayoutAccessPathEvidence, BlobLayoutCorruptionBehavior, BlobLayoutScopeSafeAbsenceBehavior,
    BlobObjectLayoutReport, ChunkTreeLayoutReport, CompactionLayoutReport, DedupeLayoutReport,
    ReachabilityLayoutReport, ReclaimLayoutReport, RetentionLayoutReport,
    StoredChunkLookupLayoutReport, StreamingLayoutReport, StreamingResumeLayoutReport,
};
#[cfg(all(test, feature = "certification-test-authority"))]
pub(crate) use publication::test_support::publish_generation_with_bytes_and_chunk_size;
#[cfg(all(test, feature = "certification-test-authority"))]
pub(crate) use retention_reclaim::test_support::reclaim_fixture as phase25_reclaim_fixture;
#[cfg(test)]
pub(crate) use streaming::layout_runtime_case;

// --- Construction-boundary compile-fail evidence ---
#[path = "compile_fail/capsule_readiness.rs"]
#[doc(hidden)]
pub mod blob_capsule_readiness_compile_fail;
#[path = "compile_fail/integrity.rs"]
#[doc(hidden)]
pub mod blob_chunk_integrity_compile_fail;
#[path = "compile_fail/root.rs"]
#[doc(hidden)]
pub mod blob_chunk_root_compile_fail;
#[path = "compile_fail/corruption.rs"]
#[doc(hidden)]
pub mod blob_corruption_compile_fail;
#[path = "compile_fail/export_bundle.rs"]
#[doc(hidden)]
pub mod blob_export_bundle_compile_fail;
#[path = "compile_fail/generation_registry.rs"]
#[doc(hidden)]
pub mod blob_generation_registry_compile_fail;
#[cfg(feature = "certification-test-authority")]
#[path = "compile_fail/harness_execution.rs"]
#[doc(hidden)]
pub mod blob_harness_execution_compile_fail;
#[path = "compile_fail/import_readmission.rs"]
#[doc(hidden)]
pub mod blob_import_readmission_compile_fail;
#[path = "compile_fail/placement_movement.rs"]
#[doc(hidden)]
pub mod blob_placement_movement_compile_fail;
#[path = "compile_fail/publication.rs"]
#[doc(hidden)]
pub mod blob_publication_commit_compile_fail;
#[path = "compile_fail/reachability.rs"]
#[doc(hidden)]
pub mod blob_reachability_compile_fail;
#[path = "compile_fail/recovery_records.rs"]
#[doc(hidden)]
pub mod blob_recovery_records_compile_fail;
#[path = "compile_fail/retention_reclaim.rs"]
#[doc(hidden)]
pub mod blob_retention_reclaim_compile_fail;
#[path = "compile_fail/streaming_execution_authority.rs"]
#[doc(hidden)]
pub mod blob_streaming_execution_authority_compile_fail;
#[path = "compile_fail/streaming_read.rs"]
#[doc(hidden)]
pub mod blob_streaming_read_compile_fail;
mod operational_repair;
#[path = "compile_fail/security_metadata.rs"]
#[doc(hidden)]
pub mod security_metadata_compile_fail;
pub use operational_repair::{
    BlobRepairConsequence, BlobRepairConsequenceDenial, BlobRepairConsequenceOwner,
    BlobRepairConsequencePlan, BlobRepairConsequenceReceipt, BlobRepairRegionObservation,
};
