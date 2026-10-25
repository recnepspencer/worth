//! Refusals when a restored generated output's required invariants are checked.

use worth_relational::facade::mvcc::CustomInvariantNonApplicability;
use worth_relational::facade::runtime::InvariantDecisionKind;

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
    /// A required invariant ran but neither affirmed the output nor was
    /// proven untouched by it.
    RequiredInvariantDidNotPass {
        /// The receipt's verdict.
        verdict: InvariantDecisionKind,
        /// Why the rule was not applicable, when the verdict says so.
        non_applicability: Option<CustomInvariantNonApplicability>,
    },
}
