//! Installs workflow registration into the ordinary Home's program runtime.
use super::WorthQueryApplicationHome;
use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner;
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::application_installation::{
    WorthQueryWorkflowRuntimeBindingDenial, WorthQueryWorkflowVocabulary,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

impl<Schema: ApplicationSchema + 'static, Program> WorthQueryApplicationHome<Schema, Program> {
    /// Register installed workflow meaning before issuing owned sessions.
    /// Refusal preserves the original Home, installed spec and signing owner.
    pub fn retain_workflow_spec<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        mut self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        authentication: WorthQueryAuthenticationEventSigningOwner<Schema>,
    ) -> Result<Self, WorthQueryWorkflowHomeBindingRefused<Schema, Spec, Program>> {
        worth_execution::run_with_execution_stack(move || {
            match self.query.register_home_workflow(workflow, authentication) {
                Ok(()) => Ok(self),
                Err((workflow, authentication, denial)) => {
                    Err(WorthQueryWorkflowHomeBindingRefused {
                        home: self,
                        workflow,
                        authentication,
                        denial,
                    })
                }
            }
        })
    }

    /// Look up this Home's installed vocabulary by its declared workflow type.
    pub fn workflow_vocabulary<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        &self,
    ) -> Option<WorthQueryWorkflowVocabulary<'_, Schema, Spec>> {
        self.query.workflow_vocabulary()
    }

    pub fn retained_workflow_spec<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        &self,
    ) -> Option<&WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>> {
        self.query.retained_workflow_spec()
    }

    /// Support the same workflow spec on another rostered program.
    pub fn support_workflow_spec<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        &mut self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    ) -> Result<
        (),
        (
            WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
            WorthQueryWorkflowRuntimeBindingDenial,
        ),
    > {
        self.query.support_home_workflow(workflow)
    }
}

/// No binding failure replaces the original Home or drops the supplied owners.
#[must_use = "binding refusal retains the original Home, spec and signing owner"]
pub struct WorthQueryWorkflowHomeBindingRefused<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    home: WorthQueryApplicationHome<Schema, Program>,
    workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    authentication: WorthQueryAuthenticationEventSigningOwner<Schema>,
    denial: WorthQueryWorkflowRuntimeBindingDenial,
}
impl<Schema, Spec, Program> WorthQueryWorkflowHomeBindingRefused<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn denial(&self) -> &WorthQueryWorkflowRuntimeBindingDenial {
        &self.denial
    }
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationHome<Schema, Program>,
        WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        WorthQueryAuthenticationEventSigningOwner<Schema>,
        WorthQueryWorkflowRuntimeBindingDenial,
    ) {
        (self.home, self.workflow, self.authentication, self.denial)
    }
}
impl<Schema, Spec, Program> std::fmt::Debug
    for WorthQueryWorkflowHomeBindingRefused<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryWorkflowHomeBindingRefused")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}
