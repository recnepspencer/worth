//! Workflow vocabularies installed against programs the host rostered.
//!
//! Adoption moves a branch to another rostered program, and a workflow
//! definition published there must bind to that program's vocabulary. The
//! runtime therefore keeps one installed spec per supported program, named by
//! the program's authoring type, and refuses a spec for a program the host
//! never rostered.

use std::any::{Any, TypeId};

use worth_query_declaration::facade::{
    application_program::{
        ApplicationProgramDefinition, ApplicationProgramRevision, ApplicationWorkflowSpec,
    },
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

use super::{
    WorthQueryWorkflowApplicationRuntime, WorthQueryWorkflowRuntimeBindingDenial,
    WorthQueryWorkflowVocabulary,
};

/// One spec installed against one rostered program, typed by that program.
///
/// `program` is `TypeId::of::<Supported>()` and `spec` is always the
/// `WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Supported>` from
/// the same generic call, so a failed downcast would be a broken invariant,
/// never an unrostered program.
pub(super) struct WorthQuerySupportedWorkflowSpec {
    program: TypeId,
    revision: ApplicationProgramRevision,
    spec: Box<dyn Any + Send + Sync>,
}

impl<Schema, Spec, Program> WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>
where
    Schema: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = Schema> + 'static,
{
    /// Adds this runtime's workflow spec as installed against another program
    /// the host rostered.
    ///
    /// The spec must come from this host's schema and from the exact installed
    /// program the roster carries. Each program has one vocabulary, so the
    /// initial program and an already supported program are refused.
    pub fn support_workflow_spec<Supported>(
        &mut self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Supported>,
    ) -> Result<(), WorthQueryWorkflowRuntimeBindingDenial>
    where
        Supported: ApplicationProgramDefinition<Schema> + 'static,
        WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Supported>: Send + Sync,
    {
        if workflow.schema_binding() != &self.runtime.installed_schema().binding_identity() {
            return Err(WorthQueryWorkflowRuntimeBindingDenial::ForeignSchema);
        }
        if workflow.program_revision() == self.runtime.program.revision() {
            return Err(WorthQueryWorkflowRuntimeBindingDenial::AlreadySupported);
        }
        let rostered = self
            .runtime
            .supported_program::<Supported>()
            .ok_or(WorthQueryWorkflowRuntimeBindingDenial::ForeignProgram)?;
        let revision = rostered.installed_program().revision().clone();
        if workflow.program_revision() != &revision {
            return Err(WorthQueryWorkflowRuntimeBindingDenial::ForeignProgram);
        }
        // A schema binding rosters one revision per program type, so an
        // existing entry for this type already names the same revision.
        if self
            .supported
            .iter()
            .any(|supported| supported.program == TypeId::of::<Supported>())
        {
            return Err(WorthQueryWorkflowRuntimeBindingDenial::AlreadySupported);
        }
        self.runtime
            .runtime
            .workflow_coverage
            .register(workflow.adoption_coverage())
            .map_err(|_| WorthQueryWorkflowRuntimeBindingDenial::ConflictingVocabularyCoverage)?;
        self.supported.push(WorthQuerySupportedWorkflowSpec {
            program: TypeId::of::<Supported>(),
            revision,
            spec: Box::new(workflow),
        });
        Ok(())
    }

    /// The vocabulary installed against one rostered program, while the host
    /// still serves that exact revision.
    pub fn supported_vocabulary<Supported>(
        &self,
    ) -> Option<WorthQueryWorkflowVocabulary<'_, Schema, Spec, Supported>>
    where
        Supported: ApplicationProgramDefinition<Schema> + 'static,
    {
        let rostered = self.runtime.supported_program::<Supported>()?;
        let supported = self
            .supported
            .iter()
            .find(|supported| supported.program == TypeId::of::<Supported>())?;
        if &supported.revision != rostered.installed_program().revision() {
            return None;
        }
        let workflow = supported
            .spec
            .downcast_ref::<WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Supported>>()
            .expect("a supported spec is stored under its own program type");
        Some(WorthQueryWorkflowVocabulary::new(
            &self.runtime.runtime,
            workflow,
            &self.authentication,
        ))
    }
}
