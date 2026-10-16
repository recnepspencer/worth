//! Complete Bank installation custody awaiting authentic workflow vocabulary binding.
use bank_domain::schema::{ApprovedBusinessPaymentWorkflow, BankSchema};
use worth_query_host::facade::{
    admission::authentication_event::WorthQueryAuthenticationEventSigningOwner,
    application_installation::WorthQueryProgramApplicationRuntime,
    domain::WorthQueryInstalledApplicationWorkflowSpec,
    primary_graph::WorthQueryApplicationInvariantProjectionAuthority,
};

use super::{BankIdentityRuntime, InstalledBankPrincipalBinding};
use crate::{
    application_definition::BankApplication,
    approval_authentication::BankApprovalAuthenticationOwner, BankIdentityRuntimeBuildError,
};

/// A refused workflow binding retains the complete installed Bank world.
/// Replace only the installed vocabulary being submitted, or release dependent
/// operation custody and call `bind` again. Retention grants no request authority.
#[must_use = "retain the installed Bank world until workflow binding can be retried"]
pub struct BankIdentityRuntimeWorkflowBinding {
    workflow:
        WorthQueryInstalledApplicationWorkflowSpec<BankSchema, ApprovedBusinessPaymentWorkflow>,
    authentication: WorthQueryAuthenticationEventSigningOwner<BankSchema>,
    binding: InstalledBankPrincipalBinding,
    invariant_projection: WorthQueryApplicationInvariantProjectionAuthority<BankSchema>,
    approval_authentication: BankApprovalAuthenticationOwner,
    runtime: WorthQueryProgramApplicationRuntime<BankSchema, BankApplication>,
}

impl BankIdentityRuntimeWorkflowBinding {
    pub(super) fn new(
        runtime: WorthQueryProgramApplicationRuntime<BankSchema, BankApplication>,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<
            BankSchema,
            ApprovedBusinessPaymentWorkflow,
        >,
        binding: InstalledBankPrincipalBinding,
        invariant_projection: WorthQueryApplicationInvariantProjectionAuthority<BankSchema>,
        approval_authentication: BankApprovalAuthenticationOwner,
    ) -> Self {
        Self {
            authentication: approval_authentication.signing_owner(),
            workflow,
            binding,
            invariant_projection,
            approval_authentication,
            runtime,
        }
    }

    pub fn workflow_spec(
        &self,
    ) -> &WorthQueryInstalledApplicationWorkflowSpec<BankSchema, ApprovedBusinessPaymentWorkflow>
    {
        &self.workflow
    }

    /// Returns the original installed spec rather than discarding it during correction.
    pub fn replace_workflow_spec(
        &mut self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<
            BankSchema,
            ApprovedBusinessPaymentWorkflow,
        >,
    ) -> WorthQueryInstalledApplicationWorkflowSpec<BankSchema, ApprovedBusinessPaymentWorkflow>
    {
        std::mem::replace(&mut self.workflow, workflow)
    }

    /// Retries actual Query binding; every refusal returns all Bank and Query custody.
    pub fn bind(self) -> Result<BankIdentityRuntime, BankIdentityRuntimeBuildError> {
        let Self {
            workflow,
            authentication,
            binding,
            invariant_projection,
            approval_authentication,
            runtime,
        } = self;
        match runtime.retain_workflow_spec(workflow, authentication) {
            Ok(runtime) => Ok(BankIdentityRuntime {
                runtime,
                binding,
                invariant_projection,
                approval_authentication,
            }),
            Err(refusal) => {
                let (runtime, workflow, authentication, denial) = refusal.into_parts();
                Err(BankIdentityRuntimeBuildError::WorkflowRuntimeBinding {
                    binding: Box::new(Self {
                        workflow,
                        authentication,
                        binding,
                        invariant_projection,
                        approval_authentication,
                        runtime,
                    }),
                    denial,
                })
            }
        }
    }
}

impl std::fmt::Debug for BankIdentityRuntimeWorkflowBinding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BankIdentityRuntimeWorkflowBinding")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
