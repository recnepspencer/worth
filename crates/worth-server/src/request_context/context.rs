use worth_foundational::facade::DiagnosticRichnessProfile;

use super::{
    WorthServerAuthenticatedPrincipal, WorthServerBranchTarget, WorthServerWorkspaceTarget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerRequestContext {
    authenticated_principal: WorthServerAuthenticatedPrincipal,
    workspace_target: WorthServerWorkspaceTarget,
    branch_target: WorthServerBranchTarget,
    diagnostics_profile: DiagnosticRichnessProfile,
}

impl WorthServerRequestContext {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.authenticated_principal
            .owned_allocation_capacity_bytes()
            .saturating_add(self.workspace_target.owned_allocation_capacity_bytes())
            .saturating_add(self.branch_target.owned_allocation_capacity_bytes())
    }
    pub(crate) fn new(
        authenticated_principal: WorthServerAuthenticatedPrincipal,
        workspace_target: WorthServerWorkspaceTarget,
        branch_target: WorthServerBranchTarget,
        diagnostics_profile: DiagnosticRichnessProfile,
    ) -> Self {
        Self {
            authenticated_principal,
            workspace_target,
            branch_target,
            diagnostics_profile,
        }
    }

    pub fn authenticated_principal(&self) -> &WorthServerAuthenticatedPrincipal {
        &self.authenticated_principal
    }

    pub fn workspace_target(&self) -> &WorthServerWorkspaceTarget {
        &self.workspace_target
    }

    pub fn branch_target(&self) -> &WorthServerBranchTarget {
        &self.branch_target
    }

    pub fn diagnostics_profile(&self) -> DiagnosticRichnessProfile {
        self.diagnostics_profile
    }
}
