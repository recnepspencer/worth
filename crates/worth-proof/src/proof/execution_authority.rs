use super::AuthorityMarker;

/// Concrete marker for the execution resource authority.
///
/// The marker has no value or public witness issuer. The live authority and
/// leases are minted only by the execution runtime's checked construction.
#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionAuthorityMarker {}

impl AuthorityMarker for ExecutionAuthorityMarker {}
