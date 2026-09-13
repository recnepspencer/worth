//! Store-owned WAL identities consumed by fresh-process recovery cleanup.
//!
//! The recovery runtime reaches WAL facts through this physical-runtime
//! facade. The runtime therefore depends on the Store owner of the lifecycle,
//! while the WAL crate remains an implementation dependency of that owner.

pub use worth_store_wal::{
    wal_frame_integrity_scope_identity, LogSequenceNumber, WalLsnRange, WalSegmentArtifactIdentity,
    WalSegmentGeneration, WalSegmentId, WalSegmentInspection,
};
