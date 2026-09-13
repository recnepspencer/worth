use worth_relational::facade::transactions::MergeExecutionOutcome;

use super::super::*;
use super::SharedState;

pub(super) fn capture_merge_authority(
    state: &SharedState,
    target_branch: &crate::runtime::WorthQueryAdmittedBranchName,
    source_branch: &crate::runtime::WorthQueryAdmittedBranchName,
) -> Result<WorthQueryBackendMergeAuthority, WorthQueryWorkspaceError> {
    state.borrow().relational_source.with_runtime(|runtime| {
        WorthQueryBackendMergeAuthority::capture(runtime, target_branch, source_branch)
    })
}

pub(super) fn validate_merge_authority(
    state: &SharedState,
    authority: &WorthQueryBackendMergeAuthority,
) -> Result<(), WorthQueryWorkspaceError> {
    state
        .borrow()
        .relational_source
        .with_runtime(|runtime| authority.validate_against(runtime))
}

pub(super) fn execute_merge(
    state: &SharedState,
    authority: &WorthQueryBackendMergeAuthority,
    declaration: &crate::workflow::LoweredMergeWorkflowDeclaration,
) -> Result<MergeExecutionOutcome, crate::effect_lifecycle::RelationalEffectExecutionFailure> {
    if declaration.merge_request().target_branch() != authority.target_branch()
        || declaration.merge_request().source_branch() != authority.source_branch()
    {
        return Err((
            crate::effect_lifecycle::EffectExecutionDenialKind::AuthorityOverrideRejected,
            "lowered merge request does not match fixture authority".to_string(),
        )
            .into());
    }
    state
        .borrow()
        .relational_source
        .with_runtime_mut(|runtime| {
            crate::effect_lifecycle::execute_lowered_merge(runtime, declaration)
        })
}
