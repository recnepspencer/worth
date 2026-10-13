//! Native truth work prices, shared by scans and their charge proofs.
/// One occupied/live slot or keyed identity examined before reading it.
pub(crate) const EXAMINE_SLOT: u64 = 1;
/// One selected record admitted before cloning it into the answer.
pub(crate) const RESERVE_RECORD: u64 = 1;
/// One keyed record borrowed or materialized.
pub(crate) const KEYED_RECORD: u64 = 1;

/// One declared aspect binding visited and cloned.
pub(crate) const ASPECT_BINDING: u64 = 1;
/// One comparison when canonicalizing aspect names.
pub(crate) const ASPECT_COMPARISON: u64 = 1;
