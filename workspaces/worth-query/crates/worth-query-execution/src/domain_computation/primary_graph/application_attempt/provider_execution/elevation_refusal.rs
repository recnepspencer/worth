//! Admission refusals recover lifecycle facts from unconsumed candidates.
use super::super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn refuse_elevation_request<
        Operation,
        Input,
        Scope,
    >(
        &self,
        candidate: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        denial: WorthQueryAdvancementDenial,
    ) -> WorthQueryElevationRequestOutcome {
        let mut candidate = candidate.into_inner();
        let Some(binding) = candidate
            .read_set
            .admission
            .take_elevation_request_binding()
        else {
            return WorthQueryElevationRequestOutcome::Denied(
                WorthQueryApplicationCommitDenial::elevation_request_program_mismatch(),
            );
        };
        requested_outcome(denial.into_commit_outcome(), binding)
    }

    pub(in crate::domain_computation::primary_graph) fn refuse_elevation_approval<
        Operation,
        Input,
        Scope,
    >(
        &self,
        candidate: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        denial: WorthQueryAdvancementDenial,
    ) -> WorthQueryElevationApprovalOutcome {
        let mut candidate = candidate.into_inner();
        let Some(binding) = candidate
            .read_set
            .admission
            .take_elevation_approval_binding()
        else {
            return WorthQueryElevationApprovalOutcome::Indeterminate;
        };
        approved_outcome(denial.into_commit_outcome(), binding)
    }

    pub(in crate::domain_computation::primary_graph) fn refuse_elevation_close<
        Operation,
        Input,
        Scope,
    >(
        &self,
        candidate: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
        denial: WorthQueryAdvancementDenial,
    ) -> WorthQueryElevationCloseOutcome {
        let mut candidate = candidate.into_inner();
        let Some(binding) = candidate.read_set.admission.take_elevation_close_binding() else {
            return WorthQueryElevationCloseOutcome::Indeterminate;
        };
        closed_outcome(denial.into_commit_outcome(), binding)
    }

    pub(in crate::domain_computation::primary_graph) fn refuse_mandatory_review<
        Operation,
        Input,
        Scope,
    >(
        &self,
        candidate: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
        denial: WorthQueryAdvancementDenial,
    ) -> WorthQueryMandatoryReviewOutcome {
        let mut candidate = candidate.into_inner();
        let Some(binding) = candidate.read_set.admission.take_mandatory_review_binding() else {
            return WorthQueryMandatoryReviewOutcome::Indeterminate;
        };
        reviewed_outcome(denial.into_commit_outcome(), binding)
    }
}
