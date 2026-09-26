use std::marker::PhantomData;

use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryProgramApplicationRuntime;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationIdempotencyBinding, WorthQueryCapabilityRevocationProgram,
    WorthQueryDelegationActivationProgram, WorthQueryElevationApprovalOutcome,
    WorthQueryElevationApprovalProgram, WorthQueryElevationCloseOutcome,
    WorthQueryElevationCloseProgram, WorthQueryElevationRequestOutcome,
    WorthQueryElevationRequestProgram, WorthQueryMandatoryReviewOutcome,
    WorthQueryMandatoryReviewProgram,
};

/// Exact installed-program membership, admitted before specialized preparation.
pub struct WorthQueryAdmittedProgramOperation<'a, Schema, Program, Operation> {
    pub(in crate::domain_computation::primary_graph) runtime:
        &'a WorthQueryProgramApplicationRuntime<Schema, Program>,
    operation: PhantomData<fn() -> Operation>,
}

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn admit_program_operation<Operation: 'static>(
        &self,
    ) -> Result<
        WorthQueryAdmittedProgramOperation<'_, Schema, Program, Operation>,
        WorthQueryApplicationCommitDenial,
    > {
        // This token names an operation, not a mutation binding. It cannot
        // distinguish an ordinary sibling from a guarded one at commit time.
        if self
            .runtime
            .operation_requires_workflow_authority::<Operation>()
        {
            return Err(WorthQueryApplicationCommitDenial::workflow_authority_required());
        }
        let operation = std::any::TypeId::of::<Operation>();
        let required_output_source = self.program.actions().iter().any(|action| {
            action.operation_type() == operation
                && action
                    .mutation_binding_type()
                    .is_some_and(|binding| self.output_source_bindings.contains(&binding))
        });
        if !self.program.contains_operation_type::<Operation>()
            || required_output_source
            || self
                .runtime
                .installed_conditionals
                .contains_operation::<Operation>()
        {
            return Err(WorthQueryApplicationCommitDenial::application_program_required());
        }
        Ok(WorthQueryAdmittedProgramOperation {
            runtime: self,
            operation: PhantomData,
        })
    }
}

impl<Schema, Program, Operation> WorthQueryAdmittedProgramOperation<'_, Schema, Program, Operation>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn compare_and_commit_elevation_request<Input, Scope>(
        self,
        program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationRequestOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.runtime
            .runtime
            .compare_and_commit_elevation_request_for_program(program, idempotency)
    }

    pub fn compare_and_commit_elevation_approval<Input, Scope>(
        self,
        program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationApprovalOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.runtime
            .runtime
            .compare_and_commit_elevation_approval_for_program(program, idempotency, None)
    }

    pub fn compare_and_commit_elevation_close<Input, Scope>(
        self,
        program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationCloseOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.runtime
            .runtime
            .compare_and_commit_elevation_close_for_program(program, idempotency, None)
    }

    pub fn compare_and_commit_mandatory_review<Input, Scope>(
        self,
        program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryMandatoryReviewOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.runtime
            .runtime
            .compare_and_commit_mandatory_review_for_program(program, idempotency, None)
    }

    pub fn compare_and_commit_capability_delegation<Input, Scope>(
        self,
        program: WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.runtime
            .runtime
            .compare_and_commit_capability_delegation_for_program(program, idempotency)
    }

    pub fn compare_and_commit_capability_revocation<Input, Scope>(
        self,
        program: WorthQueryCapabilityRevocationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.runtime
            .runtime
            .compare_and_commit_capability_revocation_for_program(program, idempotency)
    }
}
