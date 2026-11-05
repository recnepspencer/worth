//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::computation_routing::ComputationRead;

impl<Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    #[doc(hidden)]
    pub fn attributed_for_test<Output>(
        &mut self,
        read: ComputationRead,
        call: impl FnOnce(&mut Self) -> Output,
    ) -> Output {
        Self::attributed(self, read, call)
    }
}

impl<Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    #[doc(hidden)]
    pub fn begin_computation_reads_for_test(&mut self) {
        Self::begin_computation_reads(self)
    }
}

impl DecisionReads {
    #[doc(hidden)]
    pub fn computation_for_test(&self) -> &ComputationFactAttribution {
        &self.computation
    }
}

impl DecisionReads {
    #[doc(hidden)]
    pub fn reading_mut_for_test(&mut self) -> &mut Option<ComputationRead> {
        &mut self.reading
    }
}

impl DecisionReads {
    #[doc(hidden)]
    pub fn into_expected_for_test(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            ExpectedFactKeys,
            (ObservedComputationInputs, CompletedComputationRetention),
        ),
        FailedDecisionRead,
    > {
        Self::into_expected(self, execution)
    }
}
