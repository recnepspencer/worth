//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPreparedProductPublication {
    #[doc(hidden)]
    pub fn execute_for_test(
        self, execution: worth_execution::ExecutionRequest<'_, '_>) -> Result<Outcome, Denial<NoEffectCompositePublication>> {
        Self::execute(self, execution)
    }
}
