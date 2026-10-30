//! The Phase 4 literal vectors, loaded once for every unit test that checks an
//! independent reader against them. The court retains the full vector proofs;
//! these restored unit fixtures read only some vectors,
//! and each file's own `verify` runs there, so unread items are expected here.
#![allow(
    dead_code,
    reason = "unit readers use vector subsets; the integration suite verifies every full vector"
)]

#[path = "literal_vectors/checkpoint_records.rs"]
pub(crate) mod checkpoint_records;
#[path = "literal_vectors/durable_frames.rs"]
pub(crate) mod durable_frames;
#[path = "literal_vectors/oracle.rs"]
pub(crate) mod oracle;
#[path = "literal_vectors/page_frames.rs"]
pub(crate) mod page_frames;
#[path = "literal_vectors/physical_work_obligation.rs"]
pub(crate) mod physical_work_obligation;
#[path = "literal_vectors/wal_frame.rs"]
pub(crate) mod wal_frame;
