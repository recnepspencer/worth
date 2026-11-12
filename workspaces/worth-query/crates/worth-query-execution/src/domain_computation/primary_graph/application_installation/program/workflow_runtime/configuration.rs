//! One installed workflow configuration shared by Memory and Home owners.
use super::*;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use worth_runtime_world::facade::RuntimeWorldOwnerIdentity;
mod affinity;

/// Installed workflow resources, owned with their actual runtime lifecycle.
/// Its receiver identities retain no backing, Native or provider port. The
/// signing owner retires with the enclosing lifecycle owner; this is not a retained request capability.
pub struct WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub(super) workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    pub(super) authentication: WorthQueryAuthenticationEventSigningOwner<Schema>,
    pub(super) supported: Vec<WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>>,
    owner: RuntimeWorldOwnerIdentity,
    authority: WorthQueryRuntimeAuthorityIdentity,
}

impl<Schema: ApplicationSchema, Program> WorthQueryProgramApplicationRuntime<Schema, Program> {
    /// Install once before retaining runtime handles. Refusal returns both inputs.
    pub fn install_workflow_configuration<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        &mut self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        authentication: WorthQueryAuthenticationEventSigningOwner<Schema>,
    ) -> Result<
        WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>,
        (
            WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
            WorthQueryAuthenticationEventSigningOwner<Schema>,
            WorthQueryWorkflowRuntimeBindingDenial,
        ),
    > {
        let admitted = (|| {
            admit_initial(
                workflow.schema_binding(),
                authentication.binding_identity(),
                &self.installed_schema().binding_identity(),
                workflow.program_revision(),
                self.installed_program().revision(),
            )?;
            self.install_workflow_registration(&workflow, &authentication)
        })();
        if let Err(cause) = admitted {
            return Err((workflow, authentication, cause));
        }
        Ok(WorthQueryWorkflowRuntimeConfiguration {
            workflow,
            authentication,
            supported: Vec::new(),
            owner: self
                .runtime()
                .runtime
                .product_runtime
                .owner
                .owner_identity(),
            authority: self.runtime().runtime.authority_identity(),
        })
    }
}

fn admit_initial(
    schema: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    authentication: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    receiver: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    program: &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    receiver_program: &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
) -> Result<(), WorthQueryWorkflowRuntimeBindingDenial> {
    use WorthQueryWorkflowRuntimeBindingDenial as Denial;
    if schema != receiver {
        return Err(Denial::ForeignSchema);
    }
    if authentication != schema {
        return Err(Denial::ForeignAuthenticationOwner);
    }
    if program != receiver_program {
        return Err(Denial::ForeignProgram);
    }
    Ok(())
}

impl<Schema, Spec> WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub(super) fn admit_receiver<Program>(
        &self,
        runtime: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<(), WorthQueryWorkflowRuntimeBindingDenial> {
        affinity::admit(
            self.owner,
            self.authority,
            runtime
                .runtime()
                .runtime
                .product_runtime
                .owner
                .owner_identity(),
            runtime.runtime().runtime.authority_identity(),
        )?;
        if self.workflow.schema_binding() != &runtime.installed_schema().binding_identity() {
            return Err(WorthQueryWorkflowRuntimeBindingDenial::ForeignSchema);
        }
        Ok(())
    }
    /// Bind a request view only to this configuration's actual current receiver.
    pub fn vocabulary_for<'a, Program>(
        &'a self,
        runtime: &'a WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryWorkflowVocabulary<'a, Schema, Spec>,
        WorthQueryWorkflowRuntimeBindingDenial,
    > {
        self.admit_receiver(runtime)?;
        Ok(WorthQueryWorkflowVocabulary::new(
            runtime.runtime(),
            &self.workflow,
            &self.supported,
            &self.authentication,
        ))
    }
    pub const fn workflow_spec(&self) -> &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec> {
        &self.workflow
    }
    pub const fn authentication_owner(&self) -> &WorthQueryAuthenticationEventSigningOwner<Schema> {
        &self.authentication
    }
}
