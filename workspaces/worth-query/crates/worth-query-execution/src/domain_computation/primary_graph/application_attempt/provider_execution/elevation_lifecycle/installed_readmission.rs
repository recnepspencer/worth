//! Readmit installed lifecycle programs while returning their original binding on refusal.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::EntryCustodyScopedOps;
use crate::domain_computation::primary_graph::application_attempt::ReadmittedCommitProgram;

pub(in crate::domain_computation::primary_graph) trait InstalledReadmissionScopedOps<
    Schema: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
{
    fn readmit_elevation_request_installation<Operation, Input, Scope>(
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

    fn readmit_elevation_approval_installation<Operation, Input, Scope>(
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

    fn readmit_elevation_close_installation<Operation, Input, Scope>(
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

    fn readmit_mandatory_review_installation<Operation, Input, Scope>(
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

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> InstalledReadmissionScopedOps<Schema> for HostRuntime {}

mod court_doors_for_test;
pub use court_doors_for_test::InstalledReadmissionDoorsOps;
