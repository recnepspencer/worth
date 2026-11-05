//! Moved whole from `domain_computation::primary_graph::application_attempt::elevation_close_outcome`: the items below and every impl they own.

/// Why an approved elevation was closed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryElevationClosureKind {
    /// The elevation was revoked before it expired.
    Revoked,
    /// The elevation reached its expiry.
    Expired,
}
