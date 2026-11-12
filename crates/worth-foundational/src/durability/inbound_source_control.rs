//! Recorded restrictions on inbound sources, independent of a receiving runtime.
/// Immutable source-control meaning. Applying it requires the owning runtime's
/// actual source and Home custody; this value alone grants no control authority.
/// Declaration order is increasing restriction, so retirement cannot downgrade revocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InboundSourceControl {
    /// Refuse new messages; previously accepted work may finish.
    Retired,
    /// Also fence accepted work before its next external-completion publication.
    Revoked,
}
