//! Relational merge planning and inspection entry capability.

use std::time::Instant;

use worth_foundational::FoundationalMergeAdmissionOutcome;

#[cfg(test)]
use crate::merge::data::MergePlanningRequest;
use crate::merge::data::{
    MergePlanningArtifactCore, MergePlanningError, NormalizedRelationalMergeRequest,
    OwnerBoundMergePlanningRequest, RelationalFoundationalMergeRequest,
    RelationalMergeInspectionArtifact, RelationalMergeRequestNormalizationDenial,
};
use crate::runtime::RelationalRuntime;

pub struct MergeAccess<'runtime> {
    pub(super) runtime: &'runtime RelationalRuntime,
}

impl<'runtime> MergeAccess<'runtime> {
    #[cfg(test)]
    pub fn normalize_merge_planning_request(
        &self,
        request: MergePlanningRequest,
    ) -> Result<NormalizedRelationalMergeRequest, RelationalMergeRequestNormalizationDenial> {
        let bound = self
            .runtime
            .bind_merge_planning_request(request)
            .map_err(RelationalMergeRequestNormalizationDenial::OwnerBinding)?;
        self.normalize_bound_merge_planning_request(bound)
    }

    #[cfg(not(test))]
    pub fn normalize_merge_planning_request(
        &self,
        request: OwnerBoundMergePlanningRequest,
    ) -> Result<NormalizedRelationalMergeRequest, RelationalMergeRequestNormalizationDenial> {
        self.normalize_bound_merge_planning_request(request)
    }

    #[cfg(test)]
    pub fn normalize_merge_request(
        &self,
        request: crate::merge::data::MergeExecutionRequest,
    ) -> Result<NormalizedRelationalMergeRequest, RelationalMergeRequestNormalizationDenial> {
        let bound = self
            .runtime
            .bind_merge_execution_request(request)
            .map_err(RelationalMergeRequestNormalizationDenial::OwnerBinding)?;
        self.normalize_bound_merge_request(bound)
    }

    #[cfg(not(test))]
    pub fn normalize_merge_request(
        &self,
        request: crate::merge::data::OwnerBoundMergeExecutionRequest,
    ) -> Result<NormalizedRelationalMergeRequest, RelationalMergeRequestNormalizationDenial> {
        self.normalize_bound_merge_request(request)
    }

    pub fn lower_merge_request_to_foundational(
        &self,
        request: NormalizedRelationalMergeRequest,
    ) -> FoundationalMergeAdmissionOutcome<RelationalFoundationalMergeRequest> {
        super::request_foundational_lowering::lower_merge_request_to_foundational(request)
    }

    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    #[cfg(test)]
    pub fn inspect_history_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: MergePlanningRequest,
    ) -> Result<MergePlanningArtifactCore, MergePlanningError> {
        self.inspect_planning_scope(execution, request)
    }

    #[cfg(not(test))]
    pub fn inspect_history_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: OwnerBoundMergePlanningRequest,
    ) -> Result<MergePlanningArtifactCore, MergePlanningError> {
        self.inspect_planning_scope(execution, request)
    }

    #[cfg(test)]
    pub fn inspect_planning_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: MergePlanningRequest,
    ) -> Result<MergePlanningArtifactCore, MergePlanningError> {
        let started_at = Instant::now();
        let bound = self.bind_planning_request_for_test(request)?;
        let normalized_request = self
            .normalize_bound_merge_planning_request(bound)
            .map_err(MergePlanningError::from)?;
        self.inspect_bound_planning_scope(execution, normalized_request, started_at)
    }

    #[cfg(not(test))]
    pub fn inspect_planning_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: OwnerBoundMergePlanningRequest,
    ) -> Result<MergePlanningArtifactCore, MergePlanningError> {
        let started_at = Instant::now();
        let normalized_request = self.normalize_merge_planning_request(request)?;
        self.inspect_bound_planning_scope(execution, normalized_request, started_at)
    }

    fn inspect_bound_planning_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        normalized_request: NormalizedRelationalMergeRequest,
        started_at: Instant,
    ) -> Result<MergePlanningArtifactCore, MergePlanningError> {
        let plan = self.lower_planning_scope(execution, normalized_request)?;
        let artifact =
            super::planning_artifact::materialize_planning_artifact(execution, self.runtime, plan);
        self.runtime
            .performance_access()
            .count_merge_planning_elapsed(started_at.elapsed().as_nanos());
        Ok(artifact)
    }

    #[cfg(test)]
    pub fn inspect_execution_surface(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: MergePlanningRequest,
    ) -> Result<RelationalMergeInspectionArtifact, MergePlanningError> {
        let artifact = self.inspect_planning_scope(execution, request)?;
        Ok(RelationalMergeInspectionArtifact::from_input(
            artifact.inspection_input(),
        ))
    }

    #[cfg(not(test))]
    pub fn inspect_execution_surface(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: OwnerBoundMergePlanningRequest,
    ) -> Result<RelationalMergeInspectionArtifact, MergePlanningError> {
        let artifact = self.inspect_planning_scope(execution, request)?;
        Ok(RelationalMergeInspectionArtifact::from_input(
            artifact.inspection_input(),
        ))
    }

    fn normalize_bound_merge_planning_request(
        &self,
        request: OwnerBoundMergePlanningRequest,
    ) -> Result<NormalizedRelationalMergeRequest, RelationalMergeRequestNormalizationDenial> {
        super::request_normalization::normalize_bound_merge_planning_request(request)
    }

    fn normalize_bound_merge_request(
        &self,
        request: crate::merge::data::OwnerBoundMergeExecutionRequest,
    ) -> Result<NormalizedRelationalMergeRequest, RelationalMergeRequestNormalizationDenial> {
        super::request_normalization::normalize_bound_merge_execution_request(request)
    }

    #[cfg(test)]
    fn bind_planning_request_for_test(
        &self,
        request: MergePlanningRequest,
    ) -> Result<OwnerBoundMergePlanningRequest, MergePlanningError> {
        let source_branch = request.source_branch().clone();
        let target_branch = request.target_branch().clone();
        self.runtime
            .bind_merge_planning_request(request)
            .map_err(|denial| match denial {
                crate::merge::data::RelationalMergeRequestBindingDenial::UnknownBranch(branch)
                    if branch == source_branch =>
                {
                    MergePlanningError::MissingSourceHead { branch_id: branch }
                }
                crate::merge::data::RelationalMergeRequestBindingDenial::UnknownBranch(branch)
                    if branch == target_branch =>
                {
                    MergePlanningError::MissingTargetHead { branch_id: branch }
                }
                other => MergePlanningError::RequestNormalization(
                    RelationalMergeRequestNormalizationDenial::OwnerBinding(other),
                ),
            })
    }
}
