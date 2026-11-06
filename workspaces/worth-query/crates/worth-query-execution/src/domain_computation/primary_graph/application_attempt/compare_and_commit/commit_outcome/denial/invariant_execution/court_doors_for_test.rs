//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCommitDenial {
    #[doc(hidden)]
    pub fn invariant_execution_denied_for_test(
        stage: WorthQueryApplicationCommitDenialStage,
        failure: WorthQueryInvariantExecutionFailure,
    ) -> Self {
        Self::invariant_execution_denied(stage, failure)
    }
}
