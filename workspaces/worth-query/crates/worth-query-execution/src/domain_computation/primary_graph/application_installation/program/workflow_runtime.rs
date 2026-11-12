//! Program runtime paired with one immutable installed workflow vocabulary.

use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner;
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

use super::WorthQueryProgramApplicationRuntime;

mod authentication;
mod configuration;
pub use configuration::WorthQueryWorkflowRuntimeConfiguration;
mod registration;
mod support;
mod vocabulary;
pub(crate) use authentication::workflow_approval_authentication_intent;
pub use vocabulary::WorthQueryWorkflowVocabulary;

/// Retains the typed workflow vocabulary beside the program runtime that admitted it.
///
/// The vocabulary is installation truth, not branch-local definition or instance
/// truth. Keeping the pair typed prevents callers from rebuilding a convenient
/// vocabulary at each publication or start boundary.
pub struct WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    runtime: WorthQueryProgramApplicationRuntime<Schema, Program>,
    configuration: WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>,
}

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
{
    /// Bind vocabulary before cloning runtime handles or retaining operation owners.
    /// Every refusal returns all three authentic inputs for a later retry.
    pub fn retain_workflow_spec<Spec>(
        mut self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        authentication: WorthQueryAuthenticationEventSigningOwner<Schema>,
    ) -> Result<
        WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>,
        WorthQueryWorkflowRuntimeBindingRefusal<Schema, Spec, Program>,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        match self.install_workflow_configuration(workflow, authentication) {
            Ok(configuration) => Ok(WorthQueryWorkflowApplicationRuntime {
                runtime: self,
                configuration,
            }),
            Err((workflow, authentication, denial)) => {
                Err(WorthQueryWorkflowRuntimeBindingRefusal {
                    workflow,
                    authentication,
                    denial,
                    runtime: self,
                })
            }
        }
    }
}

impl<Schema, Spec, Program> WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub const fn program_runtime(&self) -> &WorthQueryProgramApplicationRuntime<Schema, Program> {
        &self.runtime
    }

    pub const fn workflow_spec(&self) -> &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec> {
        self.configuration.workflow_spec()
    }

    pub const fn authentication_owner(&self) -> &WorthQueryAuthenticationEventSigningOwner<Schema> {
        self.configuration.authentication_owner()
    }

    /// Every vocabulary this runtime installed. A request prepares against
    /// the one installed for the program its branch runs.
    pub fn vocabulary(&self) -> WorthQueryWorkflowVocabulary<'_, Schema, Spec> {
        self.configuration
            .vocabulary_for(&self.runtime)
            .expect("immutable installed workflow pair keeps its original receiver")
    }

    /// Install the same Spec for another genuinely rostered program.
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
            .support_workflow_spec(&mut self.runtime, workflow)
    }

    /// Close the contained native runtime without consuming its installed companions.
    /// A refusal keeps this complete configured runtime available for retry.
    pub fn close_conditional_runtime(
        &mut self,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryConditionalRuntimeInspection,
        crate::domain_computation::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    >
    where
        Program: worth_query_declaration::facade::application_program::ApplicationProgramDefinition<
            Schema,
        >,
    {
        self.runtime.close_conditional_runtime()
    }

    pub fn into_program_runtime(self) -> WorthQueryProgramApplicationRuntime<Schema, Program> {
        self.runtime
    }
}

impl<Schema, Spec, Program> std::ops::Deref
    for WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    type Target = WorthQueryProgramApplicationRuntime<Schema, Program>;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

/// Refused binding retains every consumed input for a lawful later attempt.
/// ```
/// use worth_query_execution::facade::application_installation::{
///     WorthQueryWorkflowRuntimeBindingRefusal, WorthQueryWorkflowApplicationRuntime,
/// };
/// use worth_query_declaration::facade::{application_program::ApplicationWorkflowSpec, application_schema::ApplicationSchema};
/// fn retry<S, Spec, P>(refusal: WorthQueryWorkflowRuntimeBindingRefusal<S, Spec, P>)
///     -> Result<WorthQueryWorkflowApplicationRuntime<S, Spec, P>, WorthQueryWorkflowRuntimeBindingRefusal<S, Spec, P>>
/// where S: ApplicationSchema, Spec: ApplicationWorkflowSpec<Schema = S> {
///     let (runtime, spec, signing_owner, _denial) = refusal.into_parts();
///     runtime.retain_workflow_spec(spec, signing_owner)
/// }
/// ```
/// Debug reports the denial only; it never renders the signing owner.
#[must_use = "retain the runtime, installed vocabulary and signing owner on refusal"]
pub struct WorthQueryWorkflowRuntimeBindingRefusal<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    authentication: WorthQueryAuthenticationEventSigningOwner<Schema>,
    denial: WorthQueryWorkflowRuntimeBindingDenial,
    runtime: WorthQueryProgramApplicationRuntime<Schema, Program>,
}
impl<Schema, Spec, Program> WorthQueryWorkflowRuntimeBindingRefusal<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub const fn denial(&self) -> &WorthQueryWorkflowRuntimeBindingDenial {
        &self.denial
    }
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryProgramApplicationRuntime<Schema, Program>,
        WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        WorthQueryAuthenticationEventSigningOwner<Schema>,
        WorthQueryWorkflowRuntimeBindingDenial,
    ) {
        (
            self.runtime,
            self.workflow,
            self.authentication,
            self.denial,
        )
    }
}
impl<Schema, Spec, Program> std::fmt::Debug
    for WorthQueryWorkflowRuntimeBindingRefusal<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryWorkflowRuntimeBindingRefusal")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowRuntimeBindingDenial {
    DurableStateAdmission(crate::domain_computation::execution_runtime::product_world::WorthQueryDurableStateAdmissionDenial),
    SigningOwner(super::WorthQuerySigningOwnerInstallationDenial),
    ForeignSchema,
    ForeignRuntime,
    StaleIncarnation,
    /// Installed runtime meaning cannot grow while dependent handles retain it.
    RuntimeKeepaliveRetained,
    ForeignProgram,
    ForeignAuthenticationOwner,
    /// The program already has a vocabulary on this runtime.
    AlreadySupported,
    /// This spec was already installed for the program with different node
    /// vocabulary, so adoption could not tell which one the host executes.
    ConflictingVocabularyCoverage,
}

impl std::fmt::Display for WorthQueryWorkflowRuntimeBindingDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "workflow runtime binding denied: {self:?}")
    }
}

impl std::error::Error for WorthQueryWorkflowRuntimeBindingDenial {}
