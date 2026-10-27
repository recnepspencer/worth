//! Stack-owned verification for one request, runtime and selected image.
use crate::domain_computation::primary_graph::{
    application_attempt::{
        FactMovement, WorthQueryApplicationObservedFact, WorthQuerySourceCurrentnessFailure,
    },
    structural_walk::StructuralWalkVerification,
};
use worth_relational::facade::{
    mvcc::CompanionPreflightStop, runtime::RelationalRuntime, snapshots::SnapshotHandle,
    transactions::RelationalExecutionDenialCause,
};

pub(in crate::domain_computation::primary_graph) struct RetainedFactVerification<
    'call,
    'request,
    'authority,
> {
    execution: &'call worth_execution::ExecutionRequest<'request, 'authority>,
    runtime: &'call RelationalRuntime,
    snapshot: &'call SnapshotHandle,
    control: Option<&'call dyn Fn() -> Result<(), RelationalExecutionDenialCause>>,
    walks: Option<StructuralWalkVerification<'call>>,
}
impl<'call, 'request, 'authority> RetainedFactVerification<'call, 'request, 'authority> {
    pub(in crate::domain_computation::primary_graph) fn new(
        execution: &'call worth_execution::ExecutionRequest<'request, 'authority>,
        runtime: &'call RelationalRuntime,
        snapshot: &'call SnapshotHandle,
        control: Option<&'call dyn Fn() -> Result<(), RelationalExecutionDenialCause>>,
    ) -> Self {
        Self {
            execution,
            runtime,
            snapshot,
            control,
            walks: None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn check_live(
        &self,
    ) -> Result<(), RelationalExecutionDenialCause> {
        if let Some(control) = self.control {
            control()?;
        }
        self.execution
            .check_live()
            .map_err(|stop| worth_execution::WorkCeilingDenial::Stopped(stop).into())
    }

    pub(super) fn observe(
        &mut self,
        fact: &WorthQueryApplicationObservedFact,
    ) -> Result<Result<FactMovement, WorthQuerySourceCurrentnessFailure>, CompanionPreflightStop>
    {
        fact.source_currentness_with(
            self.runtime,
            self.execution,
            self.snapshot,
            &mut self.walks,
            self.control,
        )
    }
}
