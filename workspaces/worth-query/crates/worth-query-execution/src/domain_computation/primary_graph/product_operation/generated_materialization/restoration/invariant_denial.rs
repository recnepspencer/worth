//! Refusals when a restored generated output's required invariants are checked.

/// Why the invariant evidence of a prepared output was not admitted. Nothing
/// was published.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGeneratedOutputInvariantAdmissionDenial {
    /// The evidence was recorded for a different branch.
    ForeignBranchEvidence,
    /// The evidence does not cover every required execution point.
    IncompleteOwnerEvidence,
    /// A required invariant did not run at its execution point.
    MissingRequiredInvariant,
    /// A required invariant ran at a different version.
    RequiredInvariantVersionMismatch,
    /// A required invariant ran more than once at the same version.
    DuplicateRequiredInvariant,
    /// A required invariant ran but did not pass.
    RequiredInvariantDidNotPass,
}
