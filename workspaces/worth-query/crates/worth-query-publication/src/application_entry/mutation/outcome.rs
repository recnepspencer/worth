use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationUncommitted,
    WorthQueryHistoricalApplicationCommit,
};

/// What executing an application mutation produced. `Committed`, `AlreadyCommitted`,
/// and `PreviouslyCommitted` name published commits. A `Commit(ProductUnpublished)`
/// retains performed owner effects awaiting product publication; an indeterminate
/// commit retains unresolved landing evidence. Neither is permission to rerun effects.
#[derive(Debug)]
pub enum WorthQueryApplicationMutationOutcome<Denial, Result> {
    Committed {
        receipt: WorthQueryApplicationCommitReceipt,
        result: Result,
    },
    AlreadyCommitted(WorthQueryApplicationCommitReceipt),
    /// The same intent committed earlier. The original live receipt and handler
    /// result are unavailable; observe current state through a fresh query.
    PreviouslyCommitted(WorthQueryHistoricalApplicationCommit),
    IdempotencyIntentDrift,
    DomainDenied(Denial),
    Cancelled,
    DeadlineExceeded,
    /// No published commit is reported here. Unpublished owner effects and
    /// unresolved landing retain their exact recovery posture in this outcome.
    Commit(WorthQueryApplicationUncommitted),
}

impl<Denial, Result> WorthQueryApplicationMutationOutcome<Denial, Result> {
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
