//! Installed workflow meaning remains paired with the original concrete Home.
use super::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeCloseRefused,
    WorthQueryApplicationHomeClosed,
};
use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner;
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQueryWorkflowRuntimeBindingDenial,
    WorthQueryWorkflowRuntimeConfiguration, WorthQueryWorkflowVocabulary,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;
mod close;
pub use close::{WorthQueryWorkflowHomeCloseRefused, WorthQueryWorkflowHomeClosed};

/// Workflow configuration and Home custody are consumed together at close.
/// The request vocabulary borrows this owner; it cannot remain usable after close.
/// ```compile_fail,E0505
/// use worth_query_host::facade::{WorthQueryWorkflowApplicationHome,
///     declaration::{application_schema::ApplicationSchema, application_program::ApplicationWorkflowSpec}};
/// fn close_with_live_view<S, Spec, P: 'static>(home: WorthQueryWorkflowApplicationHome<S, Spec, P>,
///     request: &worth_execution::ExecutionRequest<'_, '_>)
/// where S: ApplicationSchema, Spec: ApplicationWorkflowSpec<Schema = S> {
///     let vocabulary = home.vocabulary();
///     let _closed = home.close(request);
///     let _ = vocabulary.authentication_owner();
/// }
/// ```
pub struct WorthQueryWorkflowApplicationHome<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    configuration: WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>,
    home: WorthQueryApplicationHome<Schema, Program>,
}

impl<Schema: ApplicationSchema, Program> WorthQueryApplicationHome<Schema, Program> {
    /// Bind installed vocabulary before cloning runtime handles or retaining work.
    /// Refusal returns this original Home, vocabulary and signing owner.
    pub fn retain_workflow_spec<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        authentication: WorthQueryAuthenticationEventSigningOwner<Schema>,
    ) -> Result<
        WorthQueryWorkflowApplicationHome<Schema, Spec, Program>,
        WorthQueryWorkflowHomeBindingRefused<Schema, Spec, Program>,
    > {
        worth_execution::run_with_execution_stack(move || {
            let mut home = self;
            match home
                .query
                .install_workflow_configuration(workflow, authentication)
            {
                Ok(configuration) => Ok(WorthQueryWorkflowApplicationHome {
                    configuration,
                    home,
                }),
                Err((workflow, authentication, denial)) => {
                    Err(WorthQueryWorkflowHomeBindingRefused {
                        home,
                        workflow,
                        authentication,
                        denial,
                    })
                }
            }
        })
    }
}

impl<Schema, Spec, Program> WorthQueryWorkflowApplicationHome<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn runtime(&self) -> &WorthQueryProgramApplicationRuntime<Schema, Program> {
        self.home.runtime()
    }
    pub fn is_directory(&self) -> bool {
        self.home.is_directory()
    }
    pub fn workflow_spec(&self) -> &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec> {
        self.configuration.workflow_spec()
    }
    pub fn vocabulary(&self) -> WorthQueryWorkflowVocabulary<'_, Schema, Spec> {
        self.configuration
            .vocabulary_for(self.home.runtime())
            .expect("privately paired Home configuration keeps its original receiver")
    }
    pub fn support_workflow_spec(
        &mut self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    ) -> Result<
        (),
        (
            WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
            WorthQueryWorkflowRuntimeBindingDenial,
        ),
    > {
        self.configuration
            .support_workflow_spec(&mut self.home.query, workflow)
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
