use super::live_access::IndexAuthority;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::indexes::data::{DerivedIndexBuildOutcome, DerivedIndexBuildRequest};

use super::{failed_build_outcome, IndexProjectionSource};

impl IndexAuthority<'_> {
    pub(super) fn reconstruct_basis_image(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    ) -> DerivedIndexBuildOutcome {
        let observation = basis.observation();
        let Some(source_commit_id) = observation.commit_id() else {
            return failed_build_outcome(
                request.source_commit_id,
                Vec::new(),
                request.index_ids,
                None,
            );
        };
        if request.source_commit_id != source_commit_id {
            return failed_build_outcome(
                request.source_commit_id,
                Vec::new(),
                request.index_ids,
                Some(crate::branch::RelationalBranchBasisDenial::MixedAxis(
                    crate::branch::RelationalBranchBasisMismatchAxis::Commit,
                )),
            );
        }
        let source_branch_id = observation.identity().branch_id().clone();
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
        let version_id = observation.version_id();
        if let Err(refusal) = self.demand_build_index_image(execution, &request) {
            return refusal;
        }
        if let Err(failure) = basis.demand_all_full(execution, &self.runtime.services.symbols) {
            return super::build_residency::refused(&request, failure.into());
        }
        let projection = match self.runtime.read_truth().project_observation(&observation) {
            Ok(projection) => projection,
            Err(denial) => {
                return failed_build_outcome(
                    request.source_commit_id,
                    Vec::new(),
                    request.index_ids,
                    Some(denial),
                );
            }
        };
        let projection = IndexProjectionSource::exact(&projection)
            .expect(/* Proof: The owner-admitted snapshot/current projection is built from an exact retained root; speculative projections do not reach this index entry point. */ "owner-admitted observation projects one exact root");
        self.reconstruct_projected_indexes(
            execution,
            request,
            lane,
            (source_branch_id, version_id),
            &projection,
        )
    }
}
