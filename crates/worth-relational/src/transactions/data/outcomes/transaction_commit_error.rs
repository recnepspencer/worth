use serde::{Deserialize, Serialize};

use crate::errors::data::ErrorContext;
use crate::publication::data::PublicationError;
use crate::transactions::data::CommitLog;

use super::{CommitConflict, CommitPreparationError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitExecutionDenialKind {
    Admission,
    ResourceExhausted,
    Cancelled,
    DeadlineElapsed,
    WorkExhausted,
    ResultCapacityExceeded,
    WorkerFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitExecutionDenial {
    pub kind: CommitExecutionDenialKind,
    pub partition_identity: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionCommitError {
    Conflict {
        error: CommitConflict,
        commit_log: CommitLog,
    },
    Publication {
        error: PublicationError,
        commit_log: CommitLog,
    },
    Preparation {
        error: CommitPreparationError,
        commit_log: CommitLog,
    },
    #[serde(skip)]
    Execution {
        denial: CommitExecutionDenial,
        context: ErrorContext,
        commit_log: CommitLog,
    },
    Interrupted {
        interruption: crate::mvcc::RelationalInterruptionEvent,
        context: ErrorContext,
        commit_log: CommitLog,
    },
    #[serde(skip)]
    PublicationDenied {
        denial: crate::mvcc::RelationalPublicationDenial,
        context: ErrorContext,
        commit_log: CommitLog,
    },
    #[serde(skip)]
    PublicationDeferred {
        deferred: crate::mvcc::RelationalPublicationDeferred,
        context: ErrorContext,
        commit_log: CommitLog,
    },
    #[serde(skip)]
    PublicationFailed {
        failure: crate::mvcc::RelationalPublicationFailure,
        context: ErrorContext,
        commit_log: CommitLog,
    },
    #[serde(skip)]
    PerformedButDurabilityDeferred {
        settlement: crate::publication::data::DeferredPublicationSettlement,
        error: PublicationError,
        commit_log: CommitLog,
    },
}

impl TransactionCommitError {
    pub fn conflict(error: CommitConflict) -> Self {
        Self::Conflict {
            error,
            commit_log: CommitLog::new(),
        }
    }

    pub fn publication(error: PublicationError) -> Self {
        Self::Publication {
            error,
            commit_log: CommitLog::new(),
        }
    }

    pub fn preparation(error: CommitPreparationError) -> Self {
        Self::Preparation {
            error,
            commit_log: CommitLog::new(),
        }
    }

    pub(crate) fn execution(denial: CommitExecutionDenial) -> Self {
        Self::Execution {
            denial,
            context: ErrorContext::new(
                crate::errors::data::RelationalSubsystem::Transaction,
                crate::errors::data::ErrorOperation::Commit,
            ),
            commit_log: CommitLog::new(),
        }
    }

    pub fn interrupted(interruption: crate::mvcc::RelationalInterruptionEvent) -> Self {
        let operation = match interruption.boundary() {
            crate::mvcc::RelationalInterruptionBoundary::ProposalValidation => {
                crate::errors::data::ErrorOperation::Validate
            }
            crate::mvcc::RelationalInterruptionBoundary::CandidatePreparation => {
                crate::errors::data::ErrorOperation::Commit
            }
            _ => crate::errors::data::ErrorOperation::Publish,
        };
        Self::Interrupted {
            interruption,
            context: ErrorContext::new(
                crate::errors::data::RelationalSubsystem::Transaction,
                operation,
            ),
            commit_log: CommitLog::new(),
        }
    }

    pub(crate) fn publication_denied(denial: crate::mvcc::RelationalPublicationDenial) -> Self {
        Self::PublicationDenied {
            denial,
            context: publication_context(),
            commit_log: CommitLog::new(),
        }
    }

    pub(crate) fn publication_deferred(
        deferred: crate::mvcc::RelationalPublicationDeferred,
    ) -> Self {
        Self::PublicationDeferred {
            deferred,
            context: publication_context(),
            commit_log: CommitLog::new(),
        }
    }

    pub(crate) fn publication_failed(failure: crate::mvcc::RelationalPublicationFailure) -> Self {
        Self::PublicationFailed {
            failure,
            context: publication_context(),
            commit_log: CommitLog::new(),
        }
    }

    pub(crate) fn performed_but_durability_deferred(
        settlement: crate::publication::data::DeferredPublicationSettlement,
        cause: Self,
    ) -> Self {
        let commit_log = cause.commit_log().clone();
        let error = match cause {
            Self::Publication { error, .. } => error,
            other => PublicationError::new(
                crate::publication::bundle::PublicationStage::DurableAppend,
                other.detail(),
            ),
        };
        Self::PerformedButDurabilityDeferred {
            settlement,
            error,
            commit_log,
        }
    }

    pub fn with_commit_log(self, commit_log: CommitLog) -> Self {
        match self {
            Self::Conflict { error, .. } => Self::Conflict { error, commit_log },
            Self::Publication { error, .. } => Self::Publication { error, commit_log },
            Self::Preparation { error, .. } => Self::Preparation { error, commit_log },
            Self::Execution {
                denial, context, ..
            } => Self::Execution {
                denial,
                context,
                commit_log,
            },
            Self::Interrupted {
                interruption,
                context,
                ..
            } => Self::Interrupted {
                interruption,
                context,
                commit_log,
            },
            Self::PublicationDenied {
                denial, context, ..
            } => Self::PublicationDenied {
                denial,
                context,
                commit_log,
            },
            Self::PublicationDeferred {
                deferred, context, ..
            } => Self::PublicationDeferred {
                deferred,
                context,
                commit_log,
            },
            Self::PublicationFailed {
                failure, context, ..
            } => Self::PublicationFailed {
                failure,
                context,
                commit_log,
            },
            Self::PerformedButDurabilityDeferred {
                settlement, error, ..
            } => Self::PerformedButDurabilityDeferred {
                settlement,
                error,
                commit_log,
            },
        }
    }

    pub fn context(&self) -> &ErrorContext {
        match self {
            Self::Conflict { error, .. } => &error.context,
            Self::Publication { error, .. } => &error.context,
            Self::Preparation { error, .. } => error.context(),
            Self::Execution { context, .. } => context,
            Self::Interrupted { context, .. } => context,
            Self::PublicationDenied { context, .. }
            | Self::PublicationDeferred { context, .. }
            | Self::PublicationFailed { context, .. } => context,
            Self::PerformedButDurabilityDeferred { error, .. } => &error.context,
        }
    }

    pub fn detail(&self) -> String {
        match self {
            Self::Conflict { error, .. } => error.detail(),
            Self::Publication { error, .. } => error.detail.clone(),
            Self::Preparation { error, .. } => error.detail(),
            Self::Execution { denial, .. } => {
                format!("commit preparation execution stopped: {denial:?}")
            }
            Self::Interrupted { interruption, .. } => {
                format!("operation interrupted before effect: {interruption:?}")
            }
            Self::PublicationDenied { denial, .. } => {
                format!("branch publication denied: {denial:?}")
            }
            Self::PublicationDeferred { deferred, .. } => {
                format!("branch publication deferred: {deferred:?}")
            }
            Self::PublicationFailed { failure, .. } => {
                format!(
                    "branch publication failed before movement: {}",
                    failure.detail()
                )
            }
            Self::PerformedButDurabilityDeferred { error, .. } => error.detail.clone(),
        }
    }

    pub fn commit_log(&self) -> &CommitLog {
        match self {
            Self::Conflict { commit_log, .. } => commit_log,
            Self::Publication { commit_log, .. } => commit_log,
            Self::Preparation { commit_log, .. } => commit_log,
            Self::Execution { commit_log, .. } => commit_log,
            Self::Interrupted { commit_log, .. } => commit_log,
            Self::PublicationDenied { commit_log, .. }
            | Self::PublicationDeferred { commit_log, .. }
            | Self::PublicationFailed { commit_log, .. } => commit_log,
            Self::PerformedButDurabilityDeferred { commit_log, .. } => commit_log,
        }
    }

    pub fn performed_commit(&self) -> Option<&crate::history::data::RelationalCommitReceipt> {
        match self {
            Self::PerformedButDurabilityDeferred { settlement, .. } => Some(settlement.commit()),
            _ => None,
        }
    }

    pub fn deferred_settlement(
        &self,
    ) -> Option<&crate::publication::data::DeferredPublicationSettlement> {
        match self {
            Self::PerformedButDurabilityDeferred { settlement, .. } => Some(settlement),
            _ => None,
        }
    }

    pub fn commit_summary(&self) -> &crate::transactions::data::CommitSummary {
        self.commit_log().summary()
    }
}

impl From<CommitConflict> for TransactionCommitError {
    fn from(error: CommitConflict) -> Self {
        Self::conflict(error)
    }
}

fn publication_context() -> ErrorContext {
    ErrorContext::new(
        crate::errors::data::RelationalSubsystem::Transaction,
        crate::errors::data::ErrorOperation::Publish,
    )
}
