use super::{
    SignalConditionalDecisionCounters, SignalConditionalDecisionEvidence,
    SignalConditionalExecutionFailure,
};

/// Internal failure custody; an unwind is never converted to a successful decision.
pub(crate) enum SignalConditionalAttemptOutcome {
    Completed(Result<SignalConditionalDecisionEvidence, SignalConditionalExecutionFailure>),
    Unwound {
        payload: Box<dyn std::any::Any + Send>,
        counters: SignalConditionalDecisionCounters,
    },
}

impl SignalConditionalAttemptOutcome {
    pub(super) fn resume(
        self,
    ) -> Result<SignalConditionalDecisionEvidence, SignalConditionalExecutionFailure> {
        match self {
            Self::Completed(result) => result,
            Self::Unwound { payload, .. } => std::panic::resume_unwind(payload),
        }
    }
}
