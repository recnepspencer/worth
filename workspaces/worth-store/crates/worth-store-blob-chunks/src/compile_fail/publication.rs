//! Publication session closeout cannot be promoted to visible blob generations:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobPublicationSessionCloseout, BlobVisibleGeneration};
//!
//! fn requires_visible(_: BlobVisibleGeneration) {}
//!
//! let closeout: BlobPublicationSessionCloseout = todo!();
//! requires_visible(closeout);
//! ```
//! Publication WAL records cannot be promoted to visible blob generations:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobPublicationWalRecord, BlobVisibleGeneration};
//!
//! fn requires_visible(_: BlobVisibleGeneration) {}
//!
//! let record: BlobPublicationWalRecord = todo!();
//! requires_visible(record);
//! ```
//! Copied durable publication declarations cannot append blob publication WAL records:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPublicationWalRecord;
//! use worth_store_wal::PublicationDeclaration;
//!
//! let declaration: PublicationDeclaration = todo!();
//! let _record = BlobPublicationWalRecord::append(declaration);
//! ```
//! Raw crash labels cannot drive blob publication recovery:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobPublicationCrashPoint, BlobPublicationRecoveryReplay};
//!
//! let _replay = BlobPublicationRecoveryReplay::recover(
//!     BlobPublicationCrashPoint::AfterChunkWrite,
//! );
//! ```
//! Generic blob-publication classifications cannot drive blob publication recovery:
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     BlobPublicationRecoveryEvidence, LogicalContentDigest,
//! };
//! use worth_store_blob_chunks::BlobPublicationClassification;
//!
//! let digest: LogicalContentDigest = todo!();
//! let classification: BlobPublicationClassification = todo!();
//! let _evidence = BlobPublicationRecoveryEvidence::chunk_write_replayed(
//!     &digest,
//!     classification,
//! );
//! ```
//! Generic blob-publication classifications cannot mint pre-WAL blob replay evidence:
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     BlobPublicationPreWalReplayEvidence, LogicalContentDigest,
//! };
//! use worth_store_blob_chunks::BlobPublicationClassification;
//!
//! let digest: LogicalContentDigest = todo!();
//! let classification: BlobPublicationClassification = todo!();
//! let _evidence = BlobPublicationPreWalReplayEvidence::from_chunk_write_replay(
//!     &digest,
//!     &classification,
//! );
//! ```
//! Blob publication replay byte materializers are not public authority:
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     BlobPublicationPreWalReplayEvidence, LogicalContentDigest,
//! };
//!
//! let digest: LogicalContentDigest = todo!();
//! let _bytes = BlobPublicationPreWalReplayEvidence::chunk_write_persisted_replay_bytes(
//!     &digest,
//! );
//! ```
//! Raw pre-WAL partial-publication bytes are not a public replay authority:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPublicationPersistedBytes;
//!
//! let _bytes = BlobPublicationPersistedBytes::before_wal_append("copied-operation");
//! ```
//! Raw replay bytes alone cannot mint a replayed crash-edge witness:
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     BlobPublicationPersistedBytes, BlobPublicationReplayedCrashEdge,
//! };
//!
//! let bytes = BlobPublicationPersistedBytes::from_bytes(Vec::new());
//! let _edge = BlobPublicationReplayedCrashEdge::from_replayed_store_bytes(bytes);
//! ```
//! Replay-read source assembly is not a public authority surface:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPublicationReplayReadSource;
//!
//! let _source: BlobPublicationReplayReadSource = todo!();
//! ```
//! Raw persisted bytes cannot be readmitted as a before-WAL replay witness:
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     BlobPublicationPersistedBytes, BlobPublicationReplayReadWitness,
//! };
//!
//! let bytes = BlobPublicationPersistedBytes::from_bytes(Vec::new());
//! let _witness = BlobPublicationReplayReadWitness::readmitted_before_wal_append(bytes);
//! ```
//! Replay-read artifacts cannot be constructed outside the integrity-owned
//! recovery boundary:
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     BlobPublicationClassification, BlobPublicationPersistedBytes,
//!     BlobPublicationReplayReadArtifact,
//! };
//!
//! let bytes = BlobPublicationPersistedBytes::from_bytes(Vec::new());
//! let _artifact = BlobPublicationReplayReadArtifact::from_admitted_before_wal_read(
//!     "copied-entry",
//!     bytes,
//!     todo!(),
//!     todo!(),
//! );
//! ```
//! Replay-read records require an owner-issued replay-read artifact:
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     BlobPublicationPersistedBytes, BlobPublicationReplayReadRecord,
//!     BlobPublicationReplayReadArtifact,
//! };
//!
//! let bytes = BlobPublicationPersistedBytes::from_bytes(Vec::new());
//! let artifact: BlobPublicationReplayReadArtifact = todo!();
//! let _record = BlobPublicationReplayReadRecord::from_replay_read_artifact(artifact);
//! ```
//! Copied crash-edge representation cannot mint replayed crash-edge authority:
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     BlobPublicationCrashEdge, BlobPublicationReplayedCrashEdge,
//! };
//!
//! let _edge = BlobPublicationReplayedCrashEdge::from_recovery_replay_read(
//!     todo!(),
//!     BlobPublicationCrashEdge::before_wal_append("copied-operation"),
//! );
//! ```
//! Copied operation identity cannot directly mint a replay-read artifact:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPublicationReplayReadArtifact;
//!
//! let _artifact =
//!     BlobPublicationReplayReadArtifact::phase_test_before_wal_append("copied-operation");
//! ```
//! Copied operation identity cannot mint a lower before-WAL replay-read payload:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPublicationBeforeWalReplayRead;
//!
//! let _read =
//!     BlobPublicationBeforeWalReplayRead::phase_test_from_operation_digest("copied-operation");
//! ```
//! Lower before-WAL replay-read payloads cannot be synthesized by callers:
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     BlobPublicationBeforeWalReplayRead, BlobPublicationPersistedBytes,
//! };
//!
//! let _read = BlobPublicationBeforeWalReplayRead {
//!     persisted_bytes: BlobPublicationPersistedBytes::during_checkpoint_cutover("copied"),
//!     _seal: todo!(),
//! };
//! ```
//! Registry observations cannot be promoted to visible blob generations:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobGenerationObservation, BlobVisibleGeneration};
//!
//! fn requires_visible(_: BlobVisibleGeneration) {}
//!
//! let observation: BlobGenerationObservation<'_> = todo!();
//! requires_visible(observation);
//! ```
//! Published blob generations cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobGenerationPublished;
//!
//! let _forged = BlobGenerationPublished {
//!     object_id: todo!(),
//!     generation: todo!(),
//!     chunk_tree_root: todo!(),
//!     logical_content_digest: todo!(),
//!     classification: todo!(),
//!     publication_declaration: todo!(),
//!     replay_classification_digest: todo!(),
//!     replay_counters: todo!(),
//!     staging_identity: todo!(),
//!     security_metadata: todo!(),
//!     counters: todo!(),
//! };
//! ```
//! Visible blob generations cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobVisibleGeneration;
//!
//! let _forged = BlobVisibleGeneration {
//!     object_id: todo!(),
//!     generation: todo!(),
//!     chunk_tree_root: todo!(),
//!     logical_content_digest: todo!(),
//!     classification: todo!(),
//!     counters: todo!(),
//! };
//! ```
//! Counter receipt identities cannot be synthesized from copied strings:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPublicationCounterReceiptIdentity;
//!
//! let _forged = BlobPublicationCounterReceiptIdentity {
//!     value: "copied-counter-receipt".to_owned(),
//! };
//! ```
//! Recovery operation digests cannot be synthesized from copied strings:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPublicationRecoveryOperationDigest;
//!
//! let _forged = BlobPublicationRecoveryOperationDigest {
//!     value: "copied-operation".to_owned(),
//! };
//! ```
//! Pre-WAL replay evidence cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPublicationPreWalReplayEvidence;
//!
//! let _forged = BlobPublicationPreWalReplayEvidence {
//!     operation_digest: "copied-operation".to_owned(),
//!     classification_digest: "copied-classification".to_owned(),
//!     counters: todo!(),
//! };
//! ```
