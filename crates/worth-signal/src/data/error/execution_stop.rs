use std::fmt;

use worth_execution::ChargedBytes;
use worth_foundational::{ExecutionReport, PartitionIdentity};

use super::{SignalError, SignalLeaseDenial};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalPublicationDisposition {
    NoWork,
    WorkerLocal,
    PreparedPublication,
    Committed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalPublicationProgress {
    completed_epochs: usize,
    completed_tasks: usize,
    disposition: SignalPublicationDisposition,
}

impl Default for SignalPublicationProgress {
    fn default() -> Self {
        Self {
            completed_epochs: 0,
            completed_tasks: 0,
            disposition: SignalPublicationDisposition::NoWork,
        }
    }
}

impl SignalPublicationProgress {
    pub const fn completed_epochs(&self) -> usize {
        self.completed_epochs
    }
    pub const fn completed_tasks(&self) -> usize {
        self.completed_tasks
    }
    pub const fn disposition(&self) -> SignalPublicationDisposition {
        self.disposition
    }

    pub(crate) fn prepared_epoch(&mut self) {
        self.disposition = SignalPublicationDisposition::PreparedPublication;
    }

    pub(crate) fn stopped_epoch(&mut self, disposition: SignalPublicationDisposition) {
        self.disposition = disposition;
    }

    pub(crate) fn complete_epoch(&mut self, tasks: usize) {
        self.completed_epochs += 1;
        self.completed_tasks += tasks;
        self.disposition = SignalPublicationDisposition::Committed;
    }
}

impl From<SignalPublicationDisposition> for SignalPublicationProgress {
    fn from(disposition: SignalPublicationDisposition) -> Self {
        Self {
            disposition,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalExecutionFailure {
    Domain(Box<SignalError>),
    Cancelled,
    DeadlineElapsed,
    WorkCounterOverflow,
    WorkCeiling,
    NestedStopped,
    Panic,
    ResultCapacityExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalExecutionStopReason {
    Failure {
        identity: PartitionIdentity,
        cause: SignalExecutionFailure,
    },
    WorkExhausted {
        identity: PartitionIdentity,
    },
    Admission(SignalLeaseDenial),
    PreparationMemoryExhausted {
        identity: PartitionIdentity,
        required: Option<u64>,
        reserved: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalExecutionStop {
    reason: SignalExecutionStopReason,
    exclusive_prefix_boundary: Option<PartitionIdentity>,
    progress: SignalPublicationProgress,
    execution: ExecutionReport,
}

impl SignalExecutionStop {
    pub fn new(
        reason: SignalExecutionStopReason,
        exclusive_prefix_boundary: Option<PartitionIdentity>,
        progress: impl Into<SignalPublicationProgress>,
        execution: ExecutionReport,
    ) -> Self {
        Self {
            reason,
            exclusive_prefix_boundary,
            progress: progress.into(),
            execution,
        }
    }

    pub const fn reason(&self) -> &SignalExecutionStopReason {
        &self.reason
    }

    pub const fn exclusive_prefix_boundary(&self) -> Option<PartitionIdentity> {
        self.exclusive_prefix_boundary
    }

    pub const fn disposition(&self) -> SignalPublicationDisposition {
        self.progress.disposition()
    }

    pub const fn publication_progress(&self) -> SignalPublicationProgress {
        self.progress
    }

    pub const fn execution(&self) -> ExecutionReport {
        self.execution
    }
}

impl fmt::Display for SignalExecutionStop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} at exclusive prefix {:?}, publication {:?}, work {}, span {}",
            self.reason,
            self.exclusive_prefix_boundary,
            self.progress,
            self.execution.charged_work(),
            self.execution.charged_span()
        )
    }
}

impl ChargedBytes for SignalExecutionStop {
    fn additional_charged_bytes(&self) -> u64 {
        match &self.reason {
            SignalExecutionStopReason::Failure {
                cause: SignalExecutionFailure::Domain(error),
                ..
            } => error
                .additional_charged_bytes()
                .saturating_add(std::mem::size_of::<SignalError>() as u64),
            _ => 0,
        }
    }
}
