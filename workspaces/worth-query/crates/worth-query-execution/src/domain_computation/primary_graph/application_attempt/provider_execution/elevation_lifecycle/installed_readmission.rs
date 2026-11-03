//! Readmit installed lifecycle programs while returning their original binding on refusal.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::ReadmittedCommitProgram;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn readmit_elevation_request_installation<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
    ) -> Result<
        ReadmittedCommitProgram<
            '_,
            Schema,
            WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        >,
        WorthQueryElevationRequestOutcome,
    > {
        self.validate_commit_program(program)
    }

    pub(in crate::domain_computation::primary_graph) fn readmit_elevation_approval_installation<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
    ) -> Result<
        ReadmittedCommitProgram<
            '_,
            Schema,
            WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        >,
        WorthQueryElevationApprovalOutcome,
    > {
        self.validate_commit_program(program)
    }

    pub(in crate::domain_computation::primary_graph) fn readmit_elevation_close_installation<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
    ) -> Result<
        ReadmittedCommitProgram<
            '_,
            Schema,
            WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
        >,
        WorthQueryElevationCloseOutcome,
    > {
        self.validate_commit_program(program)
    }

    pub(in crate::domain_computation::primary_graph) fn readmit_mandatory_review_installation<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
    ) -> Result<
        ReadmittedCommitProgram<
            '_,
            Schema,
            WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
        >,
        WorthQueryMandatoryReviewOutcome,
    > {
        self.validate_commit_program(program)
    }
}
