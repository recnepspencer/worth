//! One admission and completion route for committed and applied projection data.
use super::*;
use worth_query_execution_attempt::facade::ports::{
    AdmittedOperationExecutionPort as _, ApplicationReadAttemptAdmissionPort as _,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Consumes the genuine before authorization lease and the separately
    /// sealed applied decision. No committed projection snapshot is constructed.
    pub fn begin_applied_application_read_attempt<'request, 'authority, Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: crate::domain_computation::primary_graph::invariant_projection::AppliedOperationProjection<Schema, Operation>,
        candidate: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'request,
            'authority,
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    > {
        let context = self.read_attempt_runtime_context();
        WorthQueryApplicationReadAttempt::admit_applied_projection_read_attempt(
            execution,
            context,
            admission,
            projection,
            candidate,
            allocation_policy,
        )
    }
}
