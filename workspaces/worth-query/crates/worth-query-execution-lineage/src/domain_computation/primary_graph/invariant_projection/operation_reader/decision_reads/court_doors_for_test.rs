//! Sealed test doors over this module's private members.
use super::*;

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

impl DecisionReads {
    /// The `FamilySelection` values this projection issued, in issue order.
    #[doc(hidden)]
    pub fn family_selections_for_test(
        &self,
    ) -> Vec<std::sync::Arc<crate::domain_computation::primary_graph::output_lineage::family_selection::ObservedFamilySelection>> {
        self.family_selections
            .iter()
            .flat_map(|issued| issued.values.iter().map(|(_, value)| std::sync::Arc::clone(value)))
            .collect()
    }

    /// The `SourcePublication` values this projection issued, in issue order.
    #[doc(hidden)]
    pub fn source_publications_for_test(
        &self,
    ) -> Vec<std::sync::Arc<crate::domain_computation::primary_graph::output_lineage::invalidation::ObservedSourcePublication>> {
        self.source_publications
            .iter()
            .flat_map(|issued| issued.values.iter().map(|(_, value)| std::sync::Arc::clone(value)))
            .collect()
    }
}
