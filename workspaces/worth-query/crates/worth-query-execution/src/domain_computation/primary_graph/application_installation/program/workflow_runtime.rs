//! Program runtime paired with one immutable installed workflow vocabulary.

use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

use super::WorthQueryProgramApplicationRuntime;

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
}

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
{
    pub fn retain_workflow_spec<Spec>(
        self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
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
        if workflow.program_revision() != self.program.revision() {
            return Err(WorthQueryWorkflowRuntimeBindingDenial::ForeignProgram);
        }
        Ok(WorthQueryWorkflowApplicationRuntime {
            runtime: self,
            workflow,
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
}
