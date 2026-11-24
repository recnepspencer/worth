use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::merge::data::{
    NormalizedRelationalMergeRequest, OwnerBoundMergeExecutionRequest,
    OwnerBoundMergePlanningRequest, RelationalMergeRequestNormalizationDenial,
};

/// Exact selected operands travel through planning separately from its descriptive request.
pub(super) struct BoundMergeInputs {
    pub(super) request: NormalizedRelationalMergeRequest,
    pub(super) target: AdmittedRelationalBranchBasis,
    pub(super) source: AdmittedRelationalBranchBasis,
}

pub(super) fn normalize_bound_merge_planning_request(
    request: OwnerBoundMergePlanningRequest,
) -> Result<BoundMergeInputs, RelationalMergeRequestNormalizationDenial> {
    let (raw, target, source) = request.into_parts();
    Ok(BoundMergeInputs {
        request: NormalizedRelationalMergeRequest::from_planning_request(raw)?,
        target,
        source,
    })
}

pub(super) fn normalize_bound_merge_execution_request(
    request: OwnerBoundMergeExecutionRequest,
) -> Result<BoundMergeInputs, RelationalMergeRequestNormalizationDenial> {
    let (raw, target, source) = request.into_parts();
    Ok(BoundMergeInputs {
        request: NormalizedRelationalMergeRequest::from_execution_request(raw)?,
        target,
        source,
    })
}
