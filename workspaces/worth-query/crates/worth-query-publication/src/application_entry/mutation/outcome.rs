use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationUncommitted,
    WorthQueryHistoricalApplicationCommit,
};

/// What executing an application mutation produced. `Committed`, `AlreadyCommitted`,
/// and `PreviouslyCommitted` name landed commits; every other variant means nothing landed.
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
    /// The commit did not land. A landed commit is `Committed` or
    /// `AlreadyCommitted` or `PreviouslyCommitted`.
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
