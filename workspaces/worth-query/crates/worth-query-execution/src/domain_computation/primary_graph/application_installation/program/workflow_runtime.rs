//! Program runtime paired with one immutable installed workflow vocabulary.

use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner;
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

use super::WorthQueryProgramApplicationRuntime;

mod authentication;
mod support;
mod vocabulary;
pub(crate) use authentication::workflow_approval_authentication_intent;
use support::WorthQuerySupportedWorkflowSpec;
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
    workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
    authentication: WorthQueryAuthenticationEventSigningOwner<Schema>,
    /// The same spec installed against programs this host rostered, one per
    /// program, so adopted branches keep a vocabulary for what they now run.
    supported: Vec<WorthQuerySupportedWorkflowSpec>,
}

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
{
    pub fn retain_workflow_spec<Spec>(
        mut self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        authentication: WorthQueryAuthenticationEventSigningOwner<Schema>,
    ) -> Result<
        WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>,
        WorthQueryWorkflowRuntimeBindingDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if workflow.schema_binding() != &self.runtime.installed_schema().binding_identity() {
            return Err(WorthQueryWorkflowRuntimeBindingDenial::ForeignSchema);
        }
        if authentication.binding_identity() != workflow.schema_binding() {
            return Err(WorthQueryWorkflowRuntimeBindingDenial::ForeignAuthenticationOwner);
        }
        if workflow.program_revision() != self.program.revision() {
            return Err(WorthQueryWorkflowRuntimeBindingDenial::ForeignProgram);
        }
        self.runtime
            .workflow_coverage
            .register(workflow.adoption_coverage())
            .map_err(|_| WorthQueryWorkflowRuntimeBindingDenial::ConflictingVocabularyCoverage)?;
        Ok(WorthQueryWorkflowApplicationRuntime {
            runtime: self,
            workflow,
            authentication,
            supported: Vec::new(),
        })
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

    pub const fn workflow_spec(
        &self,
    ) -> &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program> {
        &self.workflow
    }

    pub const fn authentication_owner(&self) -> &WorthQueryAuthenticationEventSigningOwner<Schema> {
        &self.authentication
    }

    /// The vocabulary installed against this runtime's initial program.
    pub const fn vocabulary(&self) -> WorthQueryWorkflowVocabulary<'_, Schema, Spec, Program> {
        WorthQueryWorkflowVocabulary::new(
            &self.runtime.runtime,
            &self.workflow,
            &self.authentication,
        )
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowRuntimeBindingDenial {
    ForeignSchema,
    ForeignProgram,
    ForeignAuthenticationOwner,
    /// The program already has a vocabulary on this runtime.
    AlreadySupported,
    /// This spec was already installed for the program with different node
    /// vocabulary, so adoption could not tell which one the host executes.
    ConflictingVocabularyCoverage,
}
