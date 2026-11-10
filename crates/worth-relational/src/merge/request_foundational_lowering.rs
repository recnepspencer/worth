use worth_foundational::{
    FoundationalMergeAdmissionOutcome, FoundationalMergeIntent, FoundationalMergeScope,
};
use worth_proof::TransitionOutcome;

use crate::merge::data::{
    MergeIntent, NormalizedRelationalMergeRequest, RelationalFoundationalMergeRequest,
    RelationalMergeScope,
};

pub(crate) fn lower_merge_request_to_foundational(
    request: NormalizedRelationalMergeRequest,
) -> FoundationalMergeAdmissionOutcome<RelationalFoundationalMergeRequest> {
    let foundational_scope = match request.scope() {
        RelationalMergeScope::FullBranch => FoundationalMergeScope::full_branch(),
    };
    let foundational_intent = match request.merge_intent() {
        MergeIntent::ReconcileIntoTarget => FoundationalMergeIntent::ReconcileIntoTarget,
    };

    TransitionOutcome::success(RelationalFoundationalMergeRequest::new(
        request,
        foundational_scope,
        foundational_intent,
    ))
}

#[cfg(test)]
mod tests;
