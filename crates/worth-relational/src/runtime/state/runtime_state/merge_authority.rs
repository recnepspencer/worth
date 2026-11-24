use super::RelationalRuntime;
use crate::merge::data::{
    MergeExecutionRequest, MergePlanningRequest, OwnerBoundMergeExecutionRequest,
    OwnerBoundMergePlanningRequest, RelationalMergeRequestBindingDenial,
};
use worth_relational_merge::internal_port::merge::data::requests::owner_bound::OwnerBoundMergeExecutionRequestOwnerPort as _;
use worth_relational_merge::internal_port::merge::data::requests::owner_bound::OwnerBoundMergePlanningRequestOwnerPort as _;

impl RelationalRuntime {
    /// Bind descriptive merge selectors to exact owner branch cells. The
    /// returned opaque request is required by production planning; raw
    /// `BranchId` values remain workflow provenance only.
    pub fn bind_merge_planning_request(
        &self,
        request: MergePlanningRequest,
    ) -> Result<OwnerBoundMergePlanningRequest, RelationalMergeRequestBindingDenial> {
        let target_binding =
            self.admitted_branch_basis_for_merge_branch(request.target_branch())?;
        let source_binding =
            self.admitted_branch_basis_for_merge_branch(request.source_branch())?;
        Ok(OwnerBoundMergePlanningRequest::new(
            request,
            target_binding,
            source_binding,
        ))
    }

    /// Bind descriptive merge selectors to exact owner branch cells for
    /// prepared execution.
    pub fn bind_merge_execution_request(
        &self,
        request: MergeExecutionRequest,
    ) -> Result<OwnerBoundMergeExecutionRequest, RelationalMergeRequestBindingDenial> {
        let target_binding =
            self.admitted_branch_basis_for_merge_branch(request.target_branch())?;
        let source_binding =
            self.admitted_branch_basis_for_merge_branch(request.source_branch())?;
        Ok(OwnerBoundMergeExecutionRequest::new(
            request,
            target_binding,
            source_binding,
        ))
    }
}
