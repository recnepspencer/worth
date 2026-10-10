use crate::ordinary::workflow::WorthQueryWorkflowCounters;
use crate::runtime::{
    WorthQueryOrdinaryAuthorityAdmission, WorthQueryOrdinaryAuthorityFamily,
    WorthQueryRuntimeError, WorthQueryWorkspace,
};

use super::{WorthQueryBranchMergeDeclaration, WorthQueryBranchMergeDeclarationIdentity};

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryBranchMergeContext {
    pub(crate) authority: WorthQueryOrdinaryAuthorityAdmission,
    pub(crate) declaration_identity: WorthQueryBranchMergeDeclarationIdentity,
}

#[derive(Debug)]
pub struct WorthQueryBranchMergeContextStop {
    error: WorthQueryRuntimeError,
    counters: WorthQueryWorkflowCounters,
}

impl WorthQueryBranchMergeContextStop {
    pub fn error(&self) -> &WorthQueryRuntimeError {
        &self.error
    }

    pub fn counters(&self) -> &WorthQueryWorkflowCounters {
        &self.counters
    }

    pub fn next_action(&self) -> super::WorthQueryBranchMergeNextAction {
        super::WorthQueryBranchMergeNextAction::ProvideCurrentBranchAuthority
    }
}

pub fn branch_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &WorthQueryWorkspace,
    declaration: &WorthQueryBranchMergeDeclaration,
) -> Result<WorthQueryBranchMergeContext, WorthQueryBranchMergeContextStop> {
    let authority = workspace
        .capture_ordinary_merge_authority(
            execution,
            declaration.admitted_source_branch(),
            declaration.admitted_target_branch(),
        )
        .map_err(|error| WorthQueryBranchMergeContextStop {
            error,
            counters: WorthQueryWorkflowCounters::context_checked(),
        })?;
    debug_assert_eq!(
        authority.family(),
        WorthQueryOrdinaryAuthorityFamily::BranchMerge
    );
    Ok(WorthQueryBranchMergeContext {
        authority,
        declaration_identity: declaration.identity().clone(),
    })
}
