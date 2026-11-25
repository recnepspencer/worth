/// Read-only evidence that one selected root commits to one complete visible
/// truth/schema/index/canonical-commit tuple.
///
/// The digest is a commitment over the root's resolved visible axes. Two
/// selected branches that report the same commitment observe the same visible
/// truth; two different commitments name different visible truths even when
/// their byte totals coincide. The commitment is evidence only: it carries no
/// capability and cannot be replayed into a root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalVisibilityCommitmentObservation {
    root_id: u64,
    digest: [u8; 32],
}

impl RelationalVisibilityCommitmentObservation {
    /// Owner-issued id of the root this commitment was taken over.
    ///
    /// Truth source: the selected branch's live root.
    pub const fn root_id(self) -> u64 {
        self.root_id
    }

    /// Commitment over the root's complete visible truth, schema, correctness
    /// index, and canonical commit.
    ///
    /// Truth source: the selected branch's live root, digested at observation
    /// time. It is not a recorded counter and not a stored field of the root.
    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

/// Correctness-index posture exposed by read-only MVCC inspection.
///
/// The single variant is deliberate: the correctness index is currently always
/// answered from authoritative storage, so inspection never reports a cached
/// or approximate posture. New variants may be added when a non-authoritative
/// path exists; until then callers may not infer one from the absence of
/// evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalCorrectnessIndexPosture {
    /// Correctness answers are served from authoritative storage.
    AuthoritativeFallback,
}

/// Operations used by the Relational owners.
pub trait RelationalVisibilityCommitmentObservationOwnerPort: Sized {
    fn new(root_id: u64, digest: [u8; 32]) -> Self;
}

impl RelationalVisibilityCommitmentObservationOwnerPort
    for RelationalVisibilityCommitmentObservation
{
    fn new(root_id: u64, digest: [u8; 32]) -> Self {
        Self { root_id, digest }
    }
}
