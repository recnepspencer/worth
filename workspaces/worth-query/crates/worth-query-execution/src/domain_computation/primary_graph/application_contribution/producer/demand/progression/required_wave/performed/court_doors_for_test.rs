//! Sealed test doors over this module's private members.
use super::*;

impl PerformedMembers {
    #[doc(hidden)]
    pub fn fresh_for_test<Schema: ApplicationSchema + 'static>(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        key: &WorthQueryOutputDemandKey,
        source: WorthQueryObservedSourceEpoch,
        runtime: &DecisionInput<'_, '_, '_, Schema>,
    ) -> Result<Option<FreshReadiness>, ProducerExecutionStop> {
        Self::fresh::<Schema>(self, request_execution, key, source, runtime)
    }
}
