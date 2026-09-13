use std::any::Any;

use super::{SignalBranchAdvanceDenial, SignalBranchAdvanceOutcome};

/// Exact owner completion retained across an unwinding post-movement boundary.
/// Only Signal can populate this carrier; extracting its result is once-only.
pub struct SignalBranchAdvanceCompletion {
    result: Option<Result<SignalBranchAdvanceOutcome, SignalBranchAdvanceDenial>>,
    unwind: Option<Box<dyn Any + Send>>,
}

impl SignalBranchAdvanceCompletion {
    pub(crate) fn returned(
        result: Result<SignalBranchAdvanceOutcome, SignalBranchAdvanceDenial>,
    ) -> Self {
        Self {
            result: Some(result),
            unwind: None,
        }
    }

    pub(crate) fn unwound(
        outcome: Option<SignalBranchAdvanceOutcome>,
        payload: Box<dyn Any + Send>,
    ) -> Self {
        Self {
            result: outcome.map(Ok),
            unwind: Some(payload),
        }
    }

    /// Transfer the exact returned owner result, if the call reached one.
    pub fn take_result(
        &mut self,
    ) -> Option<Result<SignalBranchAdvanceOutcome, SignalBranchAdvanceDenial>> {
        self.result.take()
    }

    /// Resume the original unwind after the consumer has secured any completion.
    pub fn resume_unwind(self) {
        if let Some(payload) = self.unwind {
            std::panic::resume_unwind(payload);
        }
    }

    pub(crate) fn into_result(
        mut self,
    ) -> Result<SignalBranchAdvanceOutcome, SignalBranchAdvanceDenial> {
        let result = self.take_result();
        self.resume_unwind();
        result.expect("a non-unwinding advancement returns an owner result")
    }
}

impl std::fmt::Debug for SignalBranchAdvanceCompletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalBranchAdvanceCompletion")
            .field("result", &self.result)
            .field("unwinding", &self.unwind.is_some())
            .finish()
    }
}
