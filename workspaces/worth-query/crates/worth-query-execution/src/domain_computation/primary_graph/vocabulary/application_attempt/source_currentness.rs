//! Moved whole from `domain_computation::primary_graph::application_attempt::fact::source_currentness`: the items below and every impl they own.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQuerySourceCurrentnessFailure {
    /// The fact's own recorded bound refuses the comparison.
    WorkBudgetExceeded,
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Unavailable,
    ManagedIndex(crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial),
    OutputObservation(crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop),
}
