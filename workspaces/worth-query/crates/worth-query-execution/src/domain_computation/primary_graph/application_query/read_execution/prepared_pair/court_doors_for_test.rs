//! Sealed test doors over this module's private members.
use super::*;

impl PreparedPair<'_> {
    #[doc(hidden)]
    pub fn run_for_test(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<PairOutput, MapKernelFailure<Denial>> {
        Self::run(self, execution, context)
    }
}

impl PairOutput {
    #[doc(hidden)]
    pub fn first_for_test(&self) -> &ReadOutput {
        &self.first
    }
}

impl PairOutput {
    #[doc(hidden)]
    pub fn second_for_test(&self) -> &ReadOutput {
        &self.second
    }
}
