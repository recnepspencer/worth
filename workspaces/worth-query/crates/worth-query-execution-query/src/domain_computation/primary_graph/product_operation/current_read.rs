//! Runtime current-product read confirmation.
use crate::domain_computation::authorization::WorthQueryAdmissionLapse;
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::product_operation::current_read::*;
use crate::domain_computation::primary_graph::{provider::WorthQueryApplicationBranchCommitLane, WorthQueryPrimaryGraphApplicationRuntime};
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::runtime::RelationalRuntime;
pub mod head;

pub trait CurrentReadControlRuntimeExt {
    fn confirm_in_execution<S: ApplicationSchema, T>(&self, execution: &ExecutionRequest<'_, '_>, runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>, value: T) -> Result<T, CurrentReadDenial>;
    fn confirm_in_execution_preserving<S: ApplicationSchema, T>(&self, execution: &ExecutionRequest<'_, '_>, runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>, value: T) -> Result<T, (T, CurrentReadDenial)>;
    fn confirm_in_borrowed<S: ApplicationSchema, T>(&self, execution: &ExecutionRequest<'_, '_>, runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>, lane: &WorthQueryApplicationBranchCommitLane, native: &RelationalRuntime, value: T) -> Result<T, CurrentReadDenial>;
    fn confirm_in_coordinated<S: ApplicationSchema, T>(&self, execution: &ExecutionRequest<'_, '_>, runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>, coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>, native: &RelationalRuntime, value: T) -> Result<T, CurrentReadDenial>;
    fn confirm_in_borrowed_preserving<S: ApplicationSchema, T>(&self, execution: &ExecutionRequest<'_, '_>, runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>, lane: &WorthQueryApplicationBranchCommitLane, native: &RelationalRuntime, value: T) -> Result<T, (T, CurrentReadDenial)>;
}

impl<'a> CurrentReadControlRuntimeExt for CurrentReadControl<'a> {
    fn confirm_in_execution<
        S: ApplicationSchema,
        T,
    >(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        value: T,
    ) -> Result<T, CurrentReadDenial> {
        self.confirm_in_execution_preserving(execution, runtime, value)
            .map_err(|(_, cause)| cause)
    }
    fn confirm_in_execution_preserving<
        S: ApplicationSchema,
        T,
    >(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        value: T,
    ) -> Result<T, (T, CurrentReadDenial)> {
        deliver(head::confirm_in_execution(
            runtime,
            execution,
            self.current,
            Some(self.request),
            value,
        ))
    }
    /// Native is already borrowed. Never block on an earlier commit lane.
    fn confirm_in_borrowed<
        S: ApplicationSchema,
        T,
    >(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        lane: &WorthQueryApplicationBranchCommitLane,
        native: &RelationalRuntime,
        value: T,
    ) -> Result<T, CurrentReadDenial> {
        self.confirm_in_borrowed_preserving(execution, runtime, lane, native, value)
            .map_err(|(_, cause)| cause)
    }
    /// The edit assembler already holds this occurrence's genuine lane. Do not
    /// re-enter it, and do not acquire Native from inside this borrowed check.
    fn confirm_in_coordinated<
        S: ApplicationSchema,
        T,
    >(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
        native: &RelationalRuntime,
        value: T,
    ) -> Result<T, CurrentReadDenial> {
        deliver(head::confirm_in_native(
            runtime,
            execution,
            self.current,
            Some(self.request),
            coordination,
            native,
            value,
        ))
        .map_err(|(_, cause)| cause)
    }
    fn confirm_in_borrowed_preserving<
        S: ApplicationSchema,
        T,
    >(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        lane: &WorthQueryApplicationBranchCommitLane,
        native: &RelationalRuntime,
        value: T,
    ) -> Result<T, (T, CurrentReadDenial)> {
        let result = {
            let Some(coordination) = lane.try_enter() else {
                return Err((value, CurrentReadDenial::Busy));
            };
            head::confirm_in_native(
                runtime,
                execution,
                self.current,
                Some(self.request),
                &coordination,
                native,
                value,
            )
        };
        deliver(result)
    }
}
fn deliver<T>(
    result: Result<(T, Option<WorthQueryAdmissionLapse>), (T, CurrentReadDenial)>,
) -> Result<T, (T, CurrentReadDenial)> {
    match result {
        Ok((value, None)) => Ok(value),
        Ok((value, Some(lapse))) => Err((value, CurrentReadDenial::Authorization(lapse))),
        Err(cause) => Err(cause),
    }
}
