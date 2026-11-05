//! Moved whole from `domain_computation::primary_graph::resolution`: the items below and every impl they own.

/// How strictly identity lookups are verified while resolving a principal or
/// entity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrincipalResolutionMode {
    /// Trust the identity index.
    Ordinary,
    /// Also verify each index lookup against stored records.
    Certification,
}
