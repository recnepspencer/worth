//! The document-retention host's installed workflow and authentication owner.

pub use std::time::Duration;
pub use worth_query_host::facade::declaration::application_program::ApplicationWorkflowSpec;

pub use worth_query_host::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQueryWorkflowApplicationRuntime,
    WorthQueryWorkflowVocabulary,
};
pub use worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition;
pub use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
pub use worth_query_host::facade::WorthQueryWorkflowApplicationHome;

pub use super::super::{
    programs::RetentionProgramP0,
    schema::DocumentRetentionSchema,
    workflow::{
        CertificationAuthenticationClockSource, CertificationAuthenticationOwner,
        ReviewedDocumentWorkflow,
    },
};

pub struct DocumentWorkflowRuntime<Program = RetentionProgramP0, Spec = ReviewedDocumentWorkflow>
where
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>,
{
    pub workflow: WorthQueryWorkflowApplicationRuntime<DocumentRetentionSchema, Spec, Program>,
    pub authentication: CertificationAuthenticationOwner,
    pub authentication_clock: CertificationAuthenticationClockSource,
}

impl<Program, Spec> DocumentWorkflowRuntime<Program, Spec>
where
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>,
{
    pub fn authentication(&self) -> &CertificationAuthenticationOwner {
        &self.authentication
    }

    /// Lets a certification add vocabularies for other rostered programs.
    pub fn workflow_mut(
        &mut self,
    ) -> &mut WorthQueryWorkflowApplicationRuntime<DocumentRetentionSchema, Spec, Program> {
        &mut self.workflow
    }

    pub fn advance_authentication_clock_for_test(&self, duration: Duration) {
        self.authentication_clock.advance_for_test(duration);
    }
}

impl<Program, Spec> std::ops::Deref for DocumentWorkflowRuntime<Program, Spec>
where
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>,
{
    type Target = WorthQueryWorkflowApplicationRuntime<DocumentRetentionSchema, Spec, Program>;

    fn deref(&self) -> &Self::Target {
        &self.workflow
    }
}

/// Every workflow entry names its vocabulary; this host's initial one is the
/// P0 vocabulary its workflow runtime retained at installation.
impl<'application, Program, Spec> From<&'application DocumentWorkflowRuntime<Program, Spec>>
    for WorthQueryWorkflowVocabulary<'application, DocumentRetentionSchema, Spec>
where
    Spec: worth_query_host::facade::declaration::application_program::ApplicationWorkflowSpec<
        Schema = DocumentRetentionSchema,
    >,
{
    fn from(application: &'application DocumentWorkflowRuntime<Program, Spec>) -> Self {
        application.workflow.vocabulary()
    }
}

/// What a reviewed-document journey needs from the application it runs
/// against. An in-memory workflow runtime and a workflow Home both provide it,
/// so one journey serves either owner.
pub trait DocumentWorkflowApplication {
    type Program: ApplicationProgramDefinition<DocumentRetentionSchema>;
    type Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>;
    fn program_runtime(
        &self,
    ) -> &WorthQueryProgramApplicationRuntime<DocumentRetentionSchema, Self::Program>;
    fn vocabulary(&self) -> WorthQueryWorkflowVocabulary<'_, DocumentRetentionSchema, Self::Spec>;
    fn authentication(&self) -> &CertificationAuthenticationOwner;
    fn runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<DocumentRetentionSchema> {
        self.program_runtime().runtime()
    }
}

impl<Program, Spec> DocumentWorkflowApplication for DocumentWorkflowRuntime<Program, Spec>
where
    Program: ApplicationProgramDefinition<DocumentRetentionSchema>,
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>,
{
    type Program = Program;
    type Spec = Spec;
    fn program_runtime(
        &self,
    ) -> &WorthQueryProgramApplicationRuntime<DocumentRetentionSchema, Program> {
        self.workflow.program_runtime()
    }
    fn vocabulary(&self) -> WorthQueryWorkflowVocabulary<'_, DocumentRetentionSchema, Spec> {
        self.workflow.vocabulary()
    }
    fn authentication(&self) -> &CertificationAuthenticationOwner {
        &self.authentication
    }
}

/// The same vocabulary bound to the Home that owns the runtime. A Directory
/// Home keeps its runtime, so the vocabulary is bound through the Home itself.
pub struct DocumentWorkflowHome<Program = RetentionProgramP0, Spec = ReviewedDocumentWorkflow>
where
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>,
{
    pub workflow: WorthQueryWorkflowApplicationHome<DocumentRetentionSchema, Spec, Program>,
    pub authentication: CertificationAuthenticationOwner,
    pub authentication_clock: CertificationAuthenticationClockSource,
}

impl<Program, Spec> DocumentWorkflowApplication for DocumentWorkflowHome<Program, Spec>
where
    Program: ApplicationProgramDefinition<DocumentRetentionSchema>,
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>,
{
    type Program = Program;
    type Spec = Spec;
    fn program_runtime(
        &self,
    ) -> &WorthQueryProgramApplicationRuntime<DocumentRetentionSchema, Program> {
        self.workflow.runtime()
    }
    fn vocabulary(&self) -> WorthQueryWorkflowVocabulary<'_, DocumentRetentionSchema, Spec> {
        self.workflow.vocabulary()
    }
    fn authentication(&self) -> &CertificationAuthenticationOwner {
        &self.authentication
    }
}

impl<'application, Program, Spec> From<&'application DocumentWorkflowHome<Program, Spec>>
    for WorthQueryWorkflowVocabulary<'application, DocumentRetentionSchema, Spec>
where
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>,
{
    fn from(application: &'application DocumentWorkflowHome<Program, Spec>) -> Self {
        application.workflow.vocabulary()
    }
}
