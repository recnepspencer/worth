use super::{
    SignalEvaluationPartition, SignalPartitionConditionalCompletion,
    SignalRejectedConditionalEvaluation,
};
use crate::data::conditional_execution::{
    SignalConditionalDecisionCounters, SignalConditionalDecisionEvidence,
    SignalConditionalExecutionFailure,
};
use crate::data::error::SignalError;
use crate::data::proof::SignalInvalidationExecutionReceipt;

/// One interrupted attempt retained before its original panic resumes.
/// This bounds report count, not dynamic receipt bytes or owner resource admission.
pub(crate) struct SignalPartitionConditionalUnwind {
    reason: SignalPartitionConditionalUnwindReason,
    rejected: SignalRejectedConditionalEvaluation,
}

pub(crate) enum SignalPartitionConditionalUnwindReason {
    Execution {
        counters: SignalConditionalDecisionCounters,
        observation:
            std::thread::Result<Result<Option<SignalInvalidationExecutionReceipt>, SignalError>>,
        cleanup: std::thread::Result<()>,
    },
    Observation {
        decision: Result<SignalConditionalDecisionEvidence, SignalConditionalExecutionFailure>,
        cleanup: std::thread::Result<()>,
    },
    Cleanup {
        completion: SignalPartitionConditionalCompletion,
    },
}

impl SignalPartitionConditionalUnwind {
    pub(super) fn new(
        reason: SignalPartitionConditionalUnwindReason,
        rejected: SignalRejectedConditionalEvaluation,
    ) -> Self {
        Self { reason, rejected }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        SignalPartitionConditionalUnwindReason,
        SignalRejectedConditionalEvaluation,
    ) {
        (self.reason, self.rejected)
    }
}

impl SignalEvaluationPartition {
    pub(crate) fn take_conditional_unwind(&mut self) -> Option<SignalPartitionConditionalUnwind> {
        self.pending_unwind.take()
    }
}
