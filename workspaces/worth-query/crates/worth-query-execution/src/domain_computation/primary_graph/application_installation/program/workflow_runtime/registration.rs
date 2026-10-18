//! Admit both installed owners before mutating either registry.
use super::super::signing::{self, AdmittedSigningOwner};
use super::{
    ApplicationSchema, ApplicationWorkflowSpec, WorthQueryAuthenticationEventSigningOwner,
    WorthQueryInstalledApplicationWorkflowSpec, WorthQueryProgramApplicationRuntime,
    WorthQueryWorkflowRuntimeBindingDenial,
};

impl<Schema: ApplicationSchema, Program> WorthQueryProgramApplicationRuntime<Schema, Program> {
    pub(super) fn install_workflow_registration<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        &mut self,
        workflow: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        authentication: &WorthQueryAuthenticationEventSigningOwner<Schema>,
    ) -> Result<(), WorthQueryWorkflowRuntimeBindingDenial> {
        let runtime = &mut std::sync::Arc::get_mut(&mut self.backing)
            .ok_or(WorthQueryWorkflowRuntimeBindingDenial::RuntimeKeepaliveRetained)?
            .runtime;
        let revision = *workflow.program_revision();
        signing::validate(runtime, &revision, authentication)
            .map_err(WorthQueryWorkflowRuntimeBindingDenial::SigningOwner)?;
        let signing =
            AdmittedSigningOwner::admit(&runtime.signing_owners, revision, authentication.clone())
                .map_err(WorthQueryWorkflowRuntimeBindingDenial::SigningOwner)?;
        let coverage = runtime
            .workflow_coverage
            .admit(workflow.adoption_coverage())
            .map_err(|_| WorthQueryWorkflowRuntimeBindingDenial::ConflictingVocabularyCoverage)?;
        signing.install();
        coverage.install();
        Ok(())
    }
}
