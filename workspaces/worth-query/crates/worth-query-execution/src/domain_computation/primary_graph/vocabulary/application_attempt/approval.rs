//! Moved whole from `domain_computation::primary_graph::application_attempt::workflow_transition_program::publication::approval`: the items below and every impl they own.

/// An approver's decision on a proposal. `Approve` and `Reject` record the approved and
/// rejected control outcomes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowApprovalDecision {
    Approve,
    Reject,
}

impl WorkflowApprovalDecision {
    pub(in crate::domain_computation::primary_graph) const fn outcome(
        self,
    ) -> worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome
    {
        match self {
            Self::Approve => worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Approved,
            Self::Reject => worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Rejected,
        }
    }
}
