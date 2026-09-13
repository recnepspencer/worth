//! Compile-fail boundary:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobResumeSessionAdmitted;
//! let _forged = BlobResumeSessionAdmitted {
//!     session_id: todo!(),
//!     authority_digest: String::new(),
//!     declaration: todo!(),
//!     counters: todo!(),
//! };
//! ```
//!
//! ```compile_fail
//! use worth_store_blob_chunks::BlobReplaySourceAdmission;
//! let _source = BlobReplaySourceAdmission::from_resume_checkpoint_digest("copied");
//! ```
//!
//! ```compile_fail
//! use worth_store_blob_chunks::BlobResumeCheckpoint;
//! fn requires_clone<T: Clone>() {}
//! requires_clone::<BlobResumeCheckpoint>();
//! ```
//!
//! ```compile_fail
//! use worth_store_physical_isolation::ReclaimEligibilityProof;
//! fn copied_orphan_digest_is_not_reclaim_coverage(proof: ReclaimEligibilityProof) {
//!     let _ = proof.bind_blob_orphan_identity(7);
//! }
//! ```
//!
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     BlobReplaySourceAdmission, BlobResumeReplayReadmission,
//! };
//! fn copied_authority_digest_is_not_replay_readmission(source: BlobReplaySourceAdmission) {
//!     let _ = BlobResumeReplayReadmission::from_checkpoint_source(&source, String::new());
//! }
//! ```

mod authority;
mod checkpoint;
mod counters;
mod denial;
mod identity;
mod ordinary_recovery;
mod orphan;
mod replay;
mod session;
mod states;
mod transitions;

#[cfg(test)]
mod tests;

pub use authority::{BlobResumeReadmissionAuthority, BlobResumeStoreAuthority};
pub use checkpoint::{
    BlobPersistedResumeCheckpointSource, BlobResumeCheckpoint, BlobResumeCheckpointReadmission,
};
pub use counters::BlobResumeCounterSnapshot;
pub use denial::{BlobResumeDenial, BlobResumeUnfinishedState};
pub use identity::{BlobResumeCheckpointIdentity, BlobResumeSessionId};
pub use ordinary_recovery::BlobInterruptedIngestRecovery;
pub use orphan::{BlobResumeSessionAbandoned, BlobResumeSessionReclaimed};
pub use replay::{
    BlobResumeReplay, BlobResumeReplayOutcome, BlobResumeRootPublicationReadyReadmitted,
};
pub use states::{
    BlobResumeCheckpointStateKind, BlobResumeChunkAppendStarted, BlobResumeChunkBytesDurable,
    BlobResumeChunkIntegrityAdmitted, BlobResumeFrontierCheckpointed, BlobResumeRootCandidateBuilt,
    BlobResumeRootPublicationReady, BlobResumeSessionAdmitted, BlobResumeSessionClosed,
    BlobResumeSessionDeclaration,
};
