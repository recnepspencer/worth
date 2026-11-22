//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationObservedFact {
    #[doc(hidden)]
    pub fn source_currentness_in_for_test(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<Result<FactMovement, WorthQuerySourceCurrentnessFailure>, CompanionPreflightStop>
    {
        Self::source_currentness_in(self, runtime, execution, snapshot)
    }
}

impl WorthQueryApplicationObservedFact {
    #[doc(hidden)]
    pub fn source_currentness_with_for_test<'runtime>(
        &self,
        runtime: &'runtime worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        reads: &mut Option<
            crate::domain_computation::primary_graph::source_read_verification::SourceReadVerification<
                'runtime,
            >,
        >,
        control: Option<
            &dyn Fn() -> Result<
                (),
                worth_relational::facade::transactions::RelationalExecutionDenialCause,
            >,
        >,
    ) -> Result<Result<FactMovement, WorthQuerySourceCurrentnessFailure>, CompanionPreflightStop>
    {
        Self::source_currentness_with(self, runtime, execution, snapshot, reads, control)
    }
}
