//! Workflow vocabularies installed against programs the host rostered.
//!
//! Adoption moves a branch to another rostered program, and a workflow
//! request prepared there must bind to that program's vocabulary. The runtime
//! therefore keeps one installed spec per supported program, keyed by the
//! program's installed revision, and refuses a spec for a program the host
//! never rostered or has retired.

use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

use super::WorthQueryWorkflowRuntimeBindingDenial;

impl<Schema, Spec> super::configuration::WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    /// Adds this runtime's workflow spec as installed against another program
    /// the host rostered.
    ///
    /// The spec must come from this host's schema and from an exact installed
    /// program the roster carries and has not retired. Each program has one
    /// vocabulary, so the initial program and an already supported program are
    /// refused. Every refusal returns the original installed spec alongside its denial.
    pub fn support_workflow_spec<Program>(
        &mut self,
        runtime: &mut super::WorthQueryProgramApplicationRuntime<Schema, Program>,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    ) -> Result<
        (),
        (
            WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
            WorthQueryWorkflowRuntimeBindingDenial,
        ),
    > {
        let binding = (|| {
            self.admit_receiver(runtime)?;
            if workflow.schema_binding() != &runtime.installed_schema().binding_identity() {
                return Err(WorthQueryWorkflowRuntimeBindingDenial::ForeignSchema);
            }
            let revision = workflow.program_revision();
            if revision == runtime.installed_program().revision()
                || self
                    .supported
                    .iter()
                    .any(|supported| supported.program_revision() == revision)
            {
                return Err(WorthQueryWorkflowRuntimeBindingDenial::AlreadySupported);
            }
            let rostered = runtime
                .backing
                .supported
                .iter()
                .any(|rostered| rostered.revision() == revision);
            let present = runtime
                .backing
                .runtime()
                .installed_program_support()
                .and_then(|support| support.present(revision))
                .is_some();
            if !(rostered && present) {
                return Err(WorthQueryWorkflowRuntimeBindingDenial::ForeignProgram);
            }
            runtime.install_workflow_registration(&workflow, &self.authentication)?;
            Ok(())
        })();
        if let Err(denial) = binding {
            return Err((workflow, denial));
        }
        self.supported.push(workflow);
        Ok(())
    }
}
