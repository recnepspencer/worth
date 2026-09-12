use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationOutputCorrespondence,
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
    Commit(WorthQueryApplicationCommitOutcome),
}

impl<Denial, Result> WorthQueryApplicationMutationOutcome<Denial, Result> {
    pub const fn commit_outcome(&self) -> Option<&WorthQueryApplicationCommitOutcome> {
        match self {
            Self::Commit(outcome) => Some(outcome),
            _ => None,
        }
    }

    pub const fn receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt> {
        match self {
            Self::Committed { receipt, .. } | Self::AlreadyCommitted(receipt) => Some(receipt),
            Self::Commit(WorthQueryApplicationCommitOutcome::Committed(receipt))
            | Self::Commit(WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt)) => {
                Some(receipt)
            }
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
