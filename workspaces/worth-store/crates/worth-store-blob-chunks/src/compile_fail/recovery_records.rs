//! Backend residue cannot satisfy admitted blob recovery records:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobRecoveryRecordSet;
//! use worth_store_physical_backend::{
//!     BackendCapabilityClaimWitness, BlobBackendResidueObservation,
//!     BlobBackendResidueObservationKind,
//! };
//!
//! fn requires_records(_: BlobRecoveryRecordSet) {}
//!
//! let capability: BackendCapabilityClaimWitness = todo!();
//! let residue = BlobBackendResidueObservation::from_store_backend_residue_scan(
//!     capability,
//!     BlobBackendResidueObservationKind::OrphanedPlacementResidue,
//!     "backend/object/key",
//! ).unwrap();
//! requires_records(residue);
//! ```
//! Physical manifest rows cannot satisfy blob reachability manifest authority:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobReachabilityManifestRow;
//! use worth_store_physical_format::{
//!     BlobPhysicalManifestRow, BlobPhysicalManifestRowKind,
//! };
//!
//! fn requires_reachability(_: BlobReachabilityManifestRow) {}
//!
//! let row = BlobPhysicalManifestRow::new(
//!     BlobPhysicalManifestRowKind::Reachability,
//!     "copied-row",
//!     1,
//!     true,
//! ).unwrap();
//! requires_reachability(row);
//! ```
//! Copied replay source identity cannot mint blob replay-source admission:
//! ```compile_fail
//! use worth_store_blob_chunks::{BlobReplaySourceAdmission, BlobReplaySourceKind};
//!
//! let _source = BlobReplaySourceAdmission::admit(
//!     BlobReplaySourceKind::Wal,
//!     "copied-operation-digest",
//! );
//! ```
//! Certification-test checkpoint replay identity cannot mint production blob replay admission:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobReplaySourceAdmission;
//!
//! let _source = BlobReplaySourceAdmission::from_checkpoint_replay_identity("copied");
//! ```
//! Certification-test manifest replay identity cannot mint production blob replay admission:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobReplaySourceAdmission;
//!
//! let _source = BlobReplaySourceAdmission::from_manifest_replay_identity("copied");
//! ```
//! Copied WAL envelopes cannot mint blob generation-publication recovery records:
//! ```compile_fail
//! use worth_store_wal::{
//!     BlobWalRecordEnvelope, BlobWalRecordIdentity, BlobWalRecordKind,
//!     PublicationDeclaration,
//! };
//!
//! let identity = BlobWalRecordIdentity::new(
//!     1,
//!     BlobWalRecordKind::GenerationPublication,
//! ).unwrap();
//! let durable: PublicationDeclaration = todo!();
//! let _envelope = BlobWalRecordEnvelope::from_wal_publication(
//!     identity,
//!     durable,
//!     "copied-payload-digest",
//! );
//! ```
//! Already-materialized published generations cannot mint replay publication records:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobGenerationPublicationRecord;
//!
//! let _record = BlobGenerationPublicationRecord::from_published_generation(
//!     todo!(),
//!     todo!(),
//!     todo!(),
//!     todo!(),
//! );
//! ```
//! Already-materialized session closeouts cannot mint replay checkpoint records:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobResumeSessionCheckpointRecord;
//!
//! let _record = BlobResumeSessionCheckpointRecord::from_session_closeout(
//!     todo!(),
//!     todo!(),
//! );
//! ```
//! Already-materialized reachability staging cannot mint manifest rows:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobReachabilityManifestRow;
//!
//! let _row = BlobReachabilityManifestRow::from_staging(todo!(), todo!());
//! ```
//! Already-materialized published generations cannot mint placement manifest rows:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPlacementManifestRow;
//!
//! let _row = BlobPlacementManifestRow::from_published_generation(todo!(), todo!());
//! ```
//! Physical-format traversal cannot be built from copied row scalars:
//! ```compile_fail
//! use worth_store_physical_format::BlobPhysicalManifestTraversal;
//!
//! let _traversal = BlobPhysicalManifestTraversal::from_observed_manifest_rows(
//!     "copied-reachability",
//!     1,
//!     "copied-placement",
//!     1,
//!     true,
//! );
//! ```
//! Backend manifest observation cannot be forged from copied row scalars:
//! ```compile_fail
//! use worth_store_physical_backend::BlobPhysicalManifestObservation;
//!
//! let _observation = BlobPhysicalManifestObservation::from_backend_manifest_traversal(
//!     "copied-reachability",
//!     1,
//!     "copied-placement",
//!     1,
//!     true,
//! );
//! ```
//! Physical-format observation source is not a public authority adapter:
//! ```compile_fail
//! use worth_store_physical_format::BlobPhysicalManifestObservationSource;
//! ```
//! Materialized publication/session/manifest objects cannot bypass admitted replay records:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobRecoveryRecordSet;
//!
//! let _records = BlobRecoveryRecordSet::from_admitted_sources(
//!     todo!(),
//!     todo!(),
//!     todo!(),
//! );
//! ```
//! Recovery record sets cannot be synthesized from public fields:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobRecoveryRecordSet;
//!
//! let _forged = BlobRecoveryRecordSet {
//!     publication: todo!(),
//!     resume_session: todo!(),
//!     manifest: todo!(),
//!     counters: todo!(),
//! };
//! ```
