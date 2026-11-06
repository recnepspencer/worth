use super::*;

/// Owner-issued input for one composite branch-creation attempt. The source
/// observation carries the exact source identity, reference occurrence, and
/// component basis. Creation never borrows a Signal transaction: forking a
/// component branch is not a mutation of that branch.
pub(crate) struct RuntimeWorldBranchCreationRequest<'a, 'request, 'authority> {
    /// The caller's request: a component fork is a durable owner transition
    /// that waits for, and is bounded by, the caller's request.
    execution: &'a worth_execution::ExecutionRequest<'request, 'authority>,
    source: ProductBranchObservation,
    intent: ProductBranchCreationIntent,
    cancellation: &'a RuntimeWorldCancellationToken,
    retained_source: Option<worth_relational::facade::branch::AdmittedRelationalRetainedForkSource>,
}

impl<'a, 'request, 'authority> RuntimeWorldBranchCreationRequest<'a, 'request, 'authority> {
    pub(crate) fn new(
        execution: &'a worth_execution::ExecutionRequest<'request, 'authority>,
        source: ProductBranchObservation,
        intent: ProductBranchCreationIntent,
        cancellation: &'a RuntimeWorldCancellationToken,
    ) -> Self {
        Self {
            execution,
            source,
            intent,
            cancellation,
            retained_source: None,
        }
    }

    pub(crate) fn retained(
        execution: &'a worth_execution::ExecutionRequest<'request, 'authority>,
        source: crate::branch::RetainedProductBranchCreationSource,
        intent: ProductBranchCreationIntent,
        cancellation: &'a RuntimeWorldCancellationToken,
    ) -> Self {
        Self {
            execution,
            source: source.observation,
            intent,
            cancellation,
            retained_source: Some(source.native),
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        &'a worth_execution::ExecutionRequest<'request, 'authority>,
        ProductBranchObservation,
        ProductBranchCreationIntent,
        &'a RuntimeWorldCancellationToken,
        Option<worth_relational::facade::branch::AdmittedRelationalRetainedForkSource>,
    ) {
        (
            self.execution,
            self.source,
            self.intent,
            self.cancellation,
            self.retained_source,
        )
    }
}
