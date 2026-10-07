//! Blob pressure and compaction planning inputs remain move-owned.
//!
//! Ingest pressure admission cannot be cloned.
//!
//! ```compile_fail
//! use worth_store_blob_chunks::BlobStreamingPressureAdmission;
//!
//! fn duplicate(admission: BlobStreamingPressureAdmission) {
//!     let _duplicate = admission.clone();
//! }
//! ```
//!
//! Ingest pressure admission cannot be consumed twice.
//!
//! ```compile_fail
//! use worth_store_blob_chunks::BlobStreamingPressureAdmission;
//!
//! fn consume(_: BlobStreamingPressureAdmission) {}
//!
//! fn consume_twice(admission: BlobStreamingPressureAdmission) {
//!     consume(admission);
//!     consume(admission);
//! }
//! ```
//!
//! A paced compaction intent cannot be cloned.
//!
//! ```compile_fail
//! use worth_store_blob_chunks::BlobCompactionIntent;
//!
//! fn duplicate(intent: BlobCompactionIntent) {
//!     let _duplicate = intent.clone();
//! }
//! ```
//!
//! A paced compaction intent cannot be consumed twice.
//!
//! ```compile_fail
//! use worth_store_blob_chunks::BlobCompactionIntent;
//!
//! fn consume(_: BlobCompactionIntent) {}
//!
//! fn consume_twice(intent: BlobCompactionIntent) {
//!     consume(intent);
//!     consume(intent);
//! }
//! ```
