//! Crate-boundary internal port for
//! `WorthQueryApplicationReadAttempt::complete_projected_dependencies_with_request`.
//! Never exported on `worth-query-execution::facade` or Host.

use worth_execution::ExecutionAllocationPolicy;

use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationReadAttempt,
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};

pub trait ProjectedReadSetInternalPort<Schema, Operation, Input, Scope> {
    fn complete_projected_dependencies_with_request(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    >;
}

impl<Schema, Operation, Input, Scope> ProjectedReadSetInternalPort<Schema, Operation, Input, Scope>
    for WorthQueryApplicationReadAttempt<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    fn complete_projected_dependencies_with_request(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        Self::complete_projected_dependencies_with_request(self, execution, allocation_policy)
    }
}
