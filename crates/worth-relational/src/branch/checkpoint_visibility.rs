//! Descriptive visibility for a verified original; no root or basis is issued.
use super::{
    RelationalBranchRootCaptureDenial, RelationalBranchRootDescriptor,
    RelationalBranchVisibilityCommitment, RelationalRootCorrectnessIndex,
};
use crate::history::data::CanonicalCommitEnvelope;
use worth_execution::ExecutionRequest;

pub(crate) fn from_original_commit_data(
    envelope: &CanonicalCommitEnvelope,
    descriptor: &RelationalBranchRootDescriptor,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RelationalBranchVisibilityCommitment, RelationalBranchRootCaptureDenial> {
    RelationalBranchVisibilityCommitment::for_root_for_request(
        envelope,
        *descriptor.truth_root(),
        *descriptor.schema_root(),
        RelationalRootCorrectnessIndex::AuthoritativeFallback,
        request,
    )
}
