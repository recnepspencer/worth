//! Moved whole from `domain_computation::primary_graph::conditional_operation::execution_provenance::classification`: the items below and every impl they own.

/// How the signal graph decided a wake's condition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalSignalDecision {
    /// The condition changed; the operation may run.
    Eligible,
    /// No dependency changed, so the operation is suppressed.
    DependencyUnchanged,
    /// The condition was recomputed and reverted to its previous value, so the
    /// operation is suppressed.
    RevertedClean,
    /// The condition was suppressed before computing.
    Suppressed,
    /// The decision was deferred until a later condition, time, or demand.
    Deferred,
}
