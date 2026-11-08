use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationUncommitted,
    WorthQueryRecoveredApplicationCommitReceipt,
};

/// What executing an application mutation produced. `Committed`, `AlreadyCommitted`,
/// and `RecoveredCommitted` name published commits. A `Commit(ProductUnpublished)`
/// retains performed owner effects awaiting product publication; an indeterminate
/// commit retains unresolved landing evidence. Neither is permission to rerun effects.
#[derive(Debug)]
pub enum WorthQueryApplicationMutationOutcome<Denial, Result> {
    Committed {
        receipt: WorthQueryApplicationCommitReceipt,
        result: Result,
    },
    AlreadyCommitted(WorthQueryApplicationCommitReceipt),
    /// Exact original durable meaning, without a reconstructed handler result.
    RecoveredCommitted(WorthQueryRecoveredApplicationCommitReceipt),
    IdempotencyIntentDrift,
    DomainDenied(Denial),
    Cancelled,
    DeadlineExceeded,
    /// No published commit is reported here. Unpublished owner effects and
    /// unresolved landing retain their exact recovery posture in this outcome.
    Commit(WorthQueryApplicationUncommitted),
}

impl<Denial, Result> WorthQueryApplicationMutationOutcome<Denial, Result> {
    pub(in crate::application_entry) fn from_commit_outcome(
        outcome: worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitOutcome,
    ) -> Self {
        use worth_query_execution::facade::primary_graph::WorthQueryApplicationLandedCommit;
        match outcome.landed() {
            Ok(WorthQueryApplicationLandedCommit::Live { receipt }) => {
                Self::AlreadyCommitted(receipt)
            }
            Ok(WorthQueryApplicationLandedCommit::Recovered(receipt)) => {
                Self::RecoveredCommitted(receipt)
            }
            Err(outcome) => Self::Commit(outcome),
        }
    }

    pub const fn recovered_receipt(&self) -> Option<&WorthQueryRecoveredApplicationCommitReceipt> {
        match self {
            Self::RecoveredCommitted(receipt) => Some(receipt),
            Self::Committed { .. }
            | Self::AlreadyCommitted(_)
            | Self::IdempotencyIntentDrift
            | Self::DomainDenied(_)
            | Self::Cancelled
            | Self::DeadlineExceeded
            | Self::Commit(_) => None,
        }
    }
    pub const fn commit_outcome(&self) -> Option<&WorthQueryApplicationUncommitted> {
        match self {
            Self::Commit(outcome) => Some(outcome),
            _ => None,
        }
    }

    pub const fn receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt> {
        match self {
            Self::Committed { receipt, .. } | Self::AlreadyCommitted(receipt) => Some(receipt),
            _ => None,
        }
    }

    pub const fn result(&self) -> Option<&Result> {
        match self {
            Self::Committed { result, .. } => Some(result),
            _ => None,
        }
    }
}
#[cfg(test)]
#[path = "outcome_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
