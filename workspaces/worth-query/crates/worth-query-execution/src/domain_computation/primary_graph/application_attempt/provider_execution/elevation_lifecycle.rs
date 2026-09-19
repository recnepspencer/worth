use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    approved_outcome, closed_outcome, requested_outcome, reviewed_outcome,
    validate_elevation_approval_program, validate_elevation_close_program,
    validate_elevation_request_program, validate_mandatory_review_program,
    WorthQueryApplicationCommitDenial, WorthQueryApplicationIdempotencyBinding,
    WorthQueryElevationApprovalOutcome, WorthQueryElevationApprovalProgram,
    WorthQueryElevationCloseOutcome, WorthQueryElevationCloseProgram,
    WorthQueryElevationRequestOutcome, WorthQueryElevationRequestProgram,
    WorthQueryMandatoryReviewOutcome, WorthQueryMandatoryReviewProgram,
};
use super::elevation_currentness::WorthQueryElevationCommitCurrentness;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn compare_and_commit_elevation_request<Operation, Input, Scope>(
        &self,
        program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationRequestOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if self.has_installed_application_program()
            || self
                .program_required_operations
                .contains(&std::any::TypeId::of::<Operation>())
        {
            return WorthQueryElevationRequestOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        self.compare_and_commit_elevation_request_for_program(program, idempotency)
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_elevation_request_for_program<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationRequestOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let mut program = program.into_inner();
        if validate_elevation_request_program(&program).is_err() {
            return WorthQueryElevationRequestOutcome::Denied(
                WorthQueryApplicationCommitDenial::elevation_request_program_mismatch(),
            );
        }
        let currentness = program
            .read_set
            .admission
            .elevation_request_binding()
            .map(WorthQueryElevationCommitCurrentness::request);
        let Some(binding) = program.read_set.admission.take_elevation_request_binding() else {
            return WorthQueryElevationRequestOutcome::Denied(
                WorthQueryApplicationCommitDenial::elevation_request_program_mismatch(),
            );
        };
        requested_outcome(
            self.compare_and_commit_application_inner_with_currentness(
                program,
                idempotency,
                currentness,
            ),
            binding,
        )
    }

    pub fn compare_and_commit_elevation_approval<Operation, Input, Scope>(
        &self,
        program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationApprovalOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_elevation_approval_for_program(
            program,
            idempotency,
            !self.has_installed_application_program()
                && !self
                    .program_required_operations
                    .contains(&std::any::TypeId::of::<Operation>()),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_elevation_approval_for_program<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        admitted: bool,
    ) -> WorthQueryElevationApprovalOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let mut program = program.into_inner();
        if !admitted {
            let Some(binding) = program.read_set.admission.take_elevation_approval_binding() else {
                return WorthQueryElevationApprovalOutcome::Indeterminate;
            };
            return WorthQueryElevationApprovalOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
                binding.into_requested(),
            );
        }
        if validate_elevation_approval_program(&program).is_err() {
            let binding = program
                .read_set
                .admission
                .take_elevation_approval_binding()
                .expect("typed approval programs retain their lifecycle binding");
            return WorthQueryElevationApprovalOutcome::Denied(
                WorthQueryApplicationCommitDenial::elevation_approval_program_mismatch(),
                binding.into_requested(),
            );
        }
        let currentness = program
            .read_set
            .admission
            .elevation_approval_binding()
            .map(WorthQueryElevationCommitCurrentness::approval);
        let Some(binding) = program.read_set.admission.take_elevation_approval_binding() else {
            return WorthQueryElevationApprovalOutcome::Indeterminate;
        };
        approved_outcome(
            self.compare_and_commit_application_inner_with_currentness(
                program,
                idempotency,
                currentness,
            ),
            binding,
        )
    }

    pub fn compare_and_commit_elevation_close<Operation, Input, Scope>(
        &self,
        program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationCloseOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_elevation_close_for_program(
            program,
            idempotency,
            !self.has_installed_application_program()
                && !self
                    .program_required_operations
                    .contains(&std::any::TypeId::of::<Operation>()),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_elevation_close_for_program<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        admitted: bool,
    ) -> WorthQueryElevationCloseOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let mut program = program.into_inner();
        if !admitted {
            let Some(binding) = program.read_set.admission.take_elevation_close_binding() else {
                return WorthQueryElevationCloseOutcome::Indeterminate;
            };
            return WorthQueryElevationCloseOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
                binding.into_approved(),
            );
        }
        if validate_elevation_close_program(&program).is_err() {
            let Some(binding) = program.read_set.admission.take_elevation_close_binding() else {
                return WorthQueryElevationCloseOutcome::Indeterminate;
            };
            return WorthQueryElevationCloseOutcome::Denied(
                WorthQueryApplicationCommitDenial::elevation_close_program_mismatch(),
                binding.into_approved(),
            );
        }
        let currentness = program
            .read_set
            .admission
            .elevation_close_binding()
            .map(WorthQueryElevationCommitCurrentness::close);
        let Some(binding) = program.read_set.admission.take_elevation_close_binding() else {
            return WorthQueryElevationCloseOutcome::Indeterminate;
        };
        closed_outcome(
            self.compare_and_commit_application_inner_with_currentness(
                program,
                idempotency,
                currentness,
            ),
            binding,
        )
    }

    pub fn compare_and_commit_mandatory_review<Operation, Input, Scope>(
        &self,
        program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryMandatoryReviewOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_mandatory_review_for_program(
            program,
            idempotency,
            !self.has_installed_application_program()
                && !self
                    .program_required_operations
                    .contains(&std::any::TypeId::of::<Operation>()),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_mandatory_review_for_program<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        admitted: bool,
    ) -> WorthQueryMandatoryReviewOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let mut program = program.into_inner();
        if !admitted {
            let Some(binding) = program.read_set.admission.take_mandatory_review_binding() else {
                return WorthQueryMandatoryReviewOutcome::Indeterminate;
            };
            return WorthQueryMandatoryReviewOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
                binding.into_mandatory(),
            );
        }
        if validate_mandatory_review_program(&program).is_err() {
            let Some(binding) = program.read_set.admission.take_mandatory_review_binding() else {
                return WorthQueryMandatoryReviewOutcome::Indeterminate;
            };
            return WorthQueryMandatoryReviewOutcome::Denied(
                WorthQueryApplicationCommitDenial::mandatory_review_program_mismatch(),
                binding.into_mandatory(),
            );
        }
        let Some(binding) = program.read_set.admission.take_mandatory_review_binding() else {
            return WorthQueryMandatoryReviewOutcome::Indeterminate;
        };
        reviewed_outcome(
            self.compare_and_commit_application_inner(program, idempotency),
            binding,
        )
    }
}
