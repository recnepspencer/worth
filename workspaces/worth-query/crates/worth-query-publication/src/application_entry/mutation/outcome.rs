use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationUncommitted,
};

#[derive(Debug)]
pub enum WorthQueryApplicationMutationOutcome<Denial, Result> {
    Committed {
        receipt: WorthQueryApplicationCommitReceipt,
        result: Result,
    },
    AlreadyCommitted(WorthQueryApplicationCommitReceipt),
    IdempotencyIntentDrift,
    DomainDenied(Denial),
    Cancelled,
    DeadlineExceeded,
    /// The commit did not land. A landed commit is `Committed` or
    /// `AlreadyCommitted`.
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

    pub fn output_correspondence(&self) -> Option<&WorthQueryApplicationOutputCorrespondence> {
        self.receipt()
            .map(WorthQueryApplicationCommitReceipt::output_correspondence)
    }

    pub const fn result(&self) -> Option<&Result> {
        match self {
            Self::Committed { result, .. } => Some(result),
            _ => None,
        }
    }
}
