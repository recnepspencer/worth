//! Completed decisions retain their real data phase until read-set completion.
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::{
    AppliedOperationProjection, PreparedCurrentInputs,
};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

pub(super) enum PreparedMutationProjection<Schema, Operation> {
    Committed(crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>),
    Applied {
        decision: AppliedOperationProjection<Schema, Operation>,
        pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    },
}
impl<Schema: ApplicationSchema, Operation> PreparedMutationProjection<Schema, Operation> {
    pub(super) fn prepared_current_inputs(&self) -> PreparedCurrentInputs<'_> {
        match self {
            Self::Committed(projection) => projection.prepared_current_inputs(),
            Self::Applied { decision, .. } => decision.prepared_current_inputs(),
        }
    }
    pub(super) fn begin_read_attempt<'request, 'authority, Input, Scope>(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'request, 'authority>,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<crate::domain_computation::primary_graph::WorthQueryApplicationReadAttempt<'request, 'authority, Schema, Operation, Input, Scope, crate::domain_computation::primary_graph::application_attempt::WorthQueryProjectedApplicationMutation>, crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>{
        match self {
            Self::Committed(projection) => runtime
                .begin_projected_application_read_attempt_with_request(
                    execution, admission, projection, allocation,
                ),
            Self::Applied { decision, pin } => runtime.begin_applied_application_read_attempt(
                execution, admission, decision, pin, allocation,
            ),
        }
    }
}
