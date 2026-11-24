use crate::merge::data::{
    NormalizedRelationalMergeRequest, OwnerBoundMergeExecutionRequest,
    OwnerBoundMergePlanningRequest, RelationalMergeRequestNormalizationDenial,
};
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;

/// Exact selected operands travel through planning separately from its descriptive request.
pub struct BoundMergeInputs {
    pub request: NormalizedRelationalMergeRequest,
    pub target: AdmittedRelationalBranchBasis,
    pub source: AdmittedRelationalBranchBasis,
}

pub fn normalize_bound_merge_planning_request(
    request: OwnerBoundMergePlanningRequest,
) -> Result<BoundMergeInputs, RelationalMergeRequestNormalizationDenial> {
    let (raw, target, source) = request.into_parts();
    Ok(BoundMergeInputs {
        request: NormalizedRelationalMergeRequest::from_planning_request(raw)?,
        target,
        source,
    })
}

pub fn normalize_bound_merge_execution_request(
    request: OwnerBoundMergeExecutionRequest,
) -> Result<BoundMergeInputs, RelationalMergeRequestNormalizationDenial> {
    let (raw, target, source) = request.into_parts();
    Ok(BoundMergeInputs {
        request: NormalizedRelationalMergeRequest::from_execution_request(raw)?,
        target,
        source,
    })
}
