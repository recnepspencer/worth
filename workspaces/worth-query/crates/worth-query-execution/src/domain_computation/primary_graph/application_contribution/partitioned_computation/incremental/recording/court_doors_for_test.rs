//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::computation_retention::full_cause::WorthQueryPartitionedComputationFullCause;

impl<Key: Send + Sync + 'static, Item> FullRecording<Key, Item> {
    #[doc(hidden)]
    pub fn complete_for_test<Reduced, Stopped>(
        self,
        results: Option<worth_execution::ExecutionRetainedRun<Reduced>>,
        tree: NativeReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>,
        execution: &crate::domain_computation::request_execution::QueryRequestExecution<'_>,
        cause: WorthQueryPartitionedComputationFullCause,
        report: ExecutionReport,
    ) -> Result<
        CompletedComputationRetention,
        super::super::super::WorthQueryPartitionedComputationDenial<Stopped>,
    >
    where
        Item: Send + Sync + 'static,
        Reduced: Send + Sync + 'static,
    {
        Self::complete::<Reduced, Stopped>(self, results, tree, execution, cause, report)
    }
}

impl<Key: Send + Sync + 'static, Item> FullRecording<Key, Item> {
    #[doc(hidden)]
    pub fn membership_for_test(
        &mut self,
        call: Option<(ComputationCallCharge, Option<u64>)>,
        items: &Items<Item>,
        digests: ItemDigests,
    ) {
        Self::membership(self, call, items, digests)
    }
}

impl<Key: Send + Sync + 'static, Item> FullRecording<Key, Item> {
    #[doc(hidden)]
    pub fn new_for_test(basis: RetainedBasisToken) -> Self {
        Self::new(basis)
    }
}
