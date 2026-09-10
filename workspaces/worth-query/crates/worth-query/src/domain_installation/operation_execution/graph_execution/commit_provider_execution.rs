use crate::domain_installation::{
    WorthQueryBoundGraphExecutionReceipt, WorthQueryGraphCommitCallRequest,
    WorthQueryGraphProviderFailure,
};

pub(super) fn contact_direct_commit_provider(
    scope_identity: &str,
    authority: &super::super::graph_participation::WorthQueryInstalledGraphCommitAuthority,
    graph_authorities: &[&worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority],
    running: &worth_query_execution::facade::runtime::WorthQueryRunningDirectRun,
) -> Result<WorthQueryBoundGraphExecutionReceipt, WorthQueryGraphProviderFailure> {
    let call = running
        .bind_commit_call(
            graph_authorities,
            WorthQueryGraphCommitCallRequest::direct(scope_identity, authority.identity()),
        )
        .map_err(|denial| WorthQueryGraphProviderFailure::new(denial.detail()))?;
    execute_commit_provider(authority, &call)
}

pub(super) fn contact_workflow_commit_provider(
    scope_identity: &str,
    stage_identity: &str,
    authority: &super::super::graph_participation::WorthQueryInstalledGraphCommitAuthority,
    graph_authorities: &[&worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority],
    running: &worth_query_execution::facade::runtime::WorthQueryRunningWorkflowRun,
) -> Result<WorthQueryBoundGraphExecutionReceipt, WorthQueryGraphProviderFailure> {
    let call = running
        .bind_stage_commit_call(
            stage_identity,
            graph_authorities,
            WorthQueryGraphCommitCallRequest::workflow_stage(
                scope_identity,
                stage_identity,
                authority.identity(),
            ),
        )
        .map_err(|denial| WorthQueryGraphProviderFailure::new(denial.detail()))?;
    execute_commit_provider(authority, &call)
}

fn execute_commit_provider(
    authority: &super::super::graph_participation::WorthQueryInstalledGraphCommitAuthority,
    call: &crate::domain_installation::WorthQueryGraphCommitCall,
) -> Result<WorthQueryBoundGraphExecutionReceipt, WorthQueryGraphProviderFailure> {
    let receipt = authority.provider.admit_commit(call)?;
    call.admit_receipt(receipt)
        .map_err(|denial| WorthQueryGraphProviderFailure::new(denial.detail()))
}
