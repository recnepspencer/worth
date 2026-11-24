//! Descriptive visibility for a verified original; no root or basis is issued.
use crate::branch::root::RelationalBranchRootCaptureDenial;
use crate::branch::root::RelationalBranchVisibilityCommitment;
use crate::branch::root::RelationalRootCorrectnessIndex;
use crate::branch::target::RelationalBranchRootDescriptor;
use crate::history::data::CanonicalCommitEnvelope;
use worth_execution::ExecutionRequest;

pub fn from_original_commit_data(
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
