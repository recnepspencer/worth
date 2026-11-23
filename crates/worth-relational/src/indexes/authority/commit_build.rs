use super::build_diagnostics::record_index_preparation_strategy_counters;
use super::build_projection::IndexBuildProjection;
use super::live_access::IndexAuthority;
use super::reconstruction_build::IndexBuildPlan;
use super::*;

impl<'runtime> IndexAuthority<'runtime> {
    pub(super) fn reconstruct_commit_image(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    ) -> DerivedIndexBuildOutcome {
        let Some((version_id, source_branch_id)) =
            self.source_commit_basis(request.source_commit_id)
        else {
            return failed_build_outcome(
                request.source_commit_id,
                Vec::new(),
                request.index_ids,
                None,
            );
        };
        if request.branch_id != source_branch_id {
            return failed_build_outcome(
                request.source_commit_id,
                Vec::new(),
                request.index_ids,
                Some(crate::branch::RelationalBranchBasisDenial::MixedAxis(
                    crate::branch::RelationalBranchBasisMismatchAxis::Branch,
                )),
            );
        }

        if let Err(refusal) = self.demand_build_index_image(execution, &request) {
            return refusal;
        }
        let plan = match self.plan_reconstruction(&request, execution) {
            Ok(plan) => plan,
            Err(refusal) => return refusal,
        };
        let mut selected_projection =
            match IndexBuildProjection::select(self.runtime, &source_branch_id, version_id) {
                Ok(projection) => projection,
                Err(denial) => return plan.unavailable(&request, Some(denial)),
            };
        if let Some(selected) = &mut selected_projection
            && let Err(cause) = selected.demand(execution)
        {
            return super::build_residency::refused(&request, cause);
        }
        let Some(projection) = selected_projection
            .as_ref()
            .and_then(IndexBuildProjection::source)
        else {
            return plan.unavailable(&request, None);
        };
        self.publish_projected_reconstruction(
            execution,
            request,
            lane,
            (source_branch_id, version_id),
            &projection,
            plan,
        )
    }

    pub(super) fn reconstruct_projected_indexes(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        selected: (
            crate::history::data::BranchId,
            crate::identity::data::VersionId,
        ),
        projection: &IndexProjectionSource<'_>,
    ) -> DerivedIndexBuildOutcome {
        let plan = match self.plan_reconstruction(&request, execution) {
            Ok(plan) => plan,
            Err(refusal) => return refusal,
        };
        self.publish_projected_reconstruction(execution, request, lane, selected, projection, plan)
    }

    fn plan_reconstruction(
        &self,
        request: &DerivedIndexBuildRequest,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<IndexBuildPlan, DerivedIndexBuildOutcome> {
        let plan = IndexBuildPlan::prepare(&self.runtime.indexes, request, execution)?;
        let (count, strategy) = plan.strategy(execution);
        record_index_preparation_strategy_counters(self.runtime, count, &strategy);
        Ok(plan)
    }

    fn publish_projected_reconstruction(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        selected: (
            crate::history::data::BranchId,
            crate::identity::data::VersionId,
        ),
        projection: &IndexProjectionSource<'_>,
        plan: IndexBuildPlan,
    ) -> DerivedIndexBuildOutcome {
        let (outcome, published) = plan.reconstruct(
            &self.runtime.indexes,
            execution,
            lane,
            &request,
            selected,
            projection,
        );
        if published {
            self.publish_build_diagnostic(
                &request,
                &request.branch_id,
                &outcome.generations,
                &outcome.failed_indexes,
            );
        }
        outcome
    }

    fn source_commit_basis(
        &self,
        commit_id: CommitId,
    ) -> Option<(
        crate::identity::data::VersionId,
        crate::history::data::BranchId,
    )> {
        self.runtime
            .history()
            .immutable_commit_receipt(commit_id)
            .map(|commit| (commit.version_id, commit.branch_id))
    }

    pub(super) fn publish_build_diagnostic(
        &self,
        request: &DerivedIndexBuildRequest,
        branch_id: &crate::history::data::BranchId,
        generations: &[DerivedIndexGeneration],
        failed_indexes: &[DerivedIndexId],
    ) {
        self.runtime
            .publication_authority()
            .push_bounded_diagnostic(
                derived_index_build_scope(),
                derived_index_build_artifact_kind(failed_indexes),
                vec![derived_index_build_completed(
                    request.source_commit_id,
                    branch_id,
                    generations,
                    failed_indexes,
                )],
            );
    }
}
