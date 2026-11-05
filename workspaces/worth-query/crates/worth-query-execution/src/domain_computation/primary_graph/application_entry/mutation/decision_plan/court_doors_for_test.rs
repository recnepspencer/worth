//! Sealed test doors over this module's private members.
use super::*;

impl<Value: 'static, Denial: 'static> WorthQueryDecisionPlan<Value, Denial> {
    #[doc(hidden)]
    pub fn run_for_test(
        self,
        execution: &QueryRequestExecution<'_>,
    ) -> HandlerResult<Value, Denial> {
        Self::run(self, execution)
    }
}
