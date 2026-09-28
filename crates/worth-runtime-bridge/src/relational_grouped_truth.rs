//! Grouped truth over Relational snapshot reads: the authoritative row set a
//! snapshot packet materializes, and the grouped projection over it. "Relational"
//! names the truth source, not the owning crate.
//!
//! This is a layer beneath the Bridge's own grouped truth view, not a copy of it:
//! [`RelationalGroupedProjectionArtifact`] implements
//! [`crate::source::GroupedProjectionSource`], which the view materializes from.
//! Its contract and digests keep names distinct from the view's so neither
//! name carries two meanings.

mod canonical_digest;
mod grouped_projection;
mod row_set;
mod snapshot_aspect_reads;

pub use grouped_projection::{
    project_relational_grouped_truth, RelationalGroupedMemberRow,
    RelationalGroupedProjectionArtifact, RelationalGroupedProjectionContract,
    RelationalGroupedProjectionDigest, RelationalGroupedTruthError,
};
pub use row_set::{
    materialize_relational_authoritative_row_set, RelationalAuthoritativeRowArtifact,
    RelationalAuthoritativeRowSetArtifact, RelationalProjectedAspectValueSet,
    RelationalRowIdentity, RelationalRowSetDigest,
};
pub use snapshot_aspect_reads::encode_snapshot_aspect_read_value;

#[cfg(test)]
mod canonical_digest_parity_tests;
