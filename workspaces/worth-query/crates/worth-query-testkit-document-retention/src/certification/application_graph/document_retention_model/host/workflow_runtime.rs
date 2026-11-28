//! The document-retention host's installed workflow and authentication owner.

pub use std::time::Duration;
pub use worth_query_host::facade::declaration::application_program::ApplicationWorkflowSpec;

pub use worth_query_host::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQueryWorkflowApplicationRuntime,
    WorthQueryWorkflowVocabulary,
};
pub use worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition;
pub use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
pub use worth_query_host::facade::WorthQueryApplicationHome;

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
pub struct DocumentWorkflowHome<Program = RetentionProgramP0> {
    pub workflow: WorthQueryApplicationHome<DocumentRetentionSchema, Program>,
    pub authentication: CertificationAuthenticationOwner,
    pub authentication_clock: CertificationAuthenticationClockSource,
}

impl<Program> DocumentWorkflowApplication for DocumentWorkflowHome<Program>
where
    Program: ApplicationProgramDefinition<DocumentRetentionSchema>,
{
    type Program = Program;
    type Spec = ReviewedDocumentWorkflow;
    fn program_runtime(
        &self,
    ) -> &WorthQueryProgramApplicationRuntime<DocumentRetentionSchema, Program> {
        self.workflow.runtime()
    }
    fn vocabulary(
        &self,
    ) -> WorthQueryWorkflowVocabulary<'_, DocumentRetentionSchema, ReviewedDocumentWorkflow> {
        self.workflow
            .workflow_vocabulary()
            .expect("workflow registered")
    }
    fn authentication(&self) -> &CertificationAuthenticationOwner {
        &self.authentication
    }
}

impl<'application, Program> From<&'application DocumentWorkflowHome<Program>>
    for WorthQueryWorkflowVocabulary<
        'application,
        DocumentRetentionSchema,
        ReviewedDocumentWorkflow,
    >
{
    fn from(application: &'application DocumentWorkflowHome<Program>) -> Self {
        application
            .workflow
            .workflow_vocabulary()
            .expect("workflow registered")
    }
}

/// RAII Directory Home wrapper owning a `WorthQueryApplicationHome` and
/// its temporary directory on disk so certification tests close and reopen cleanly.
pub struct DocumentWorkflowDirectoryHome<
    Program: 'static = RetentionProgramP0,
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema> = ReviewedDocumentWorkflow,
> {
    workflow: Option<WorthQueryApplicationHome<DocumentRetentionSchema, Program>>,
    pub authentication: CertificationAuthenticationOwner,
    pub authentication_clock: CertificationAuthenticationClockSource,
    directory: std::path::PathBuf,
    cleanup_dir: bool,
    spec: std::marker::PhantomData<fn() -> Spec>,
}

impl<Program: 'static, Spec> DocumentWorkflowDirectoryHome<Program, Spec>
where
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>,
{
    pub fn wrap(
        workflow: WorthQueryApplicationHome<DocumentRetentionSchema, Program>,
        authentication: CertificationAuthenticationOwner,
        authentication_clock: CertificationAuthenticationClockSource,
        directory: std::path::PathBuf,
    ) -> Self {
        Self {
            workflow: Some(workflow),
            authentication,
            authentication_clock,
            directory,
            cleanup_dir: true,
            spec: std::marker::PhantomData,
        }
    }

    pub fn home(&self) -> &WorthQueryApplicationHome<DocumentRetentionSchema, Program> {
        self.workflow
            .as_ref()
            .expect("workflow directory Home is open")
    }

    pub fn home_mut(&mut self) -> &mut WorthQueryApplicationHome<DocumentRetentionSchema, Program> {
        self.workflow
            .as_mut()
            .expect("workflow directory Home is open")
    }

    pub fn directory(&self) -> &std::path::Path {
        &self.directory
    }

    pub fn workflow_spec(
        &self,
    ) -> &worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
        DocumentRetentionSchema,
        Spec,
    > {
        self.home()
            .retained_workflow_spec()
            .expect("workflow registered")
    }

    pub fn close_for_reopen(mut self) -> std::path::PathBuf {
        self.cleanup_dir = false;
        let workflow = self
            .workflow
            .take()
            .expect("workflow directory Home is open");
        let lease =
            worth_query_testkit_document_retention::facade::directory_home::directory_lease();
        workflow
            .close(&worth_execution::ExecutionRequest::leased(&lease))
            .unwrap_or_else(|refused| panic!("workflow directory Home close refused: {refused:?}"))
            .reclaim();
        self.directory.clone()
    }
}

impl<Program, Spec> DocumentWorkflowApplication for DocumentWorkflowDirectoryHome<Program, Spec>
where
    Program: ApplicationProgramDefinition<DocumentRetentionSchema> + 'static,
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>,
{
    type Program = Program;
    type Spec = Spec;
    fn program_runtime(
        &self,
    ) -> &WorthQueryProgramApplicationRuntime<DocumentRetentionSchema, Program> {
        self.home().runtime()
    }
    fn vocabulary(&self) -> WorthQueryWorkflowVocabulary<'_, DocumentRetentionSchema, Spec> {
        self.home()
            .workflow_vocabulary()
            .expect("workflow registered")
    }
    fn authentication(&self) -> &CertificationAuthenticationOwner {
        &self.authentication
    }
}

impl<'application, Program: 'static, Spec>
    From<&'application DocumentWorkflowDirectoryHome<Program, Spec>>
    for WorthQueryWorkflowVocabulary<'application, DocumentRetentionSchema, Spec>
where
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>,
{
    fn from(application: &'application DocumentWorkflowDirectoryHome<Program, Spec>) -> Self {
        application
            .home()
            .workflow_vocabulary()
            .expect("workflow registered")
    }
}

impl<Program: 'static, Spec> Drop for DocumentWorkflowDirectoryHome<Program, Spec>
where
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>,
{
    fn drop(&mut self) {
        if let Some(workflow) = self.workflow.take() {
            let lease =
                worth_query_testkit_document_retention::facade::directory_home::directory_lease();
            if let Ok(closed) = workflow.close(&worth_execution::ExecutionRequest::leased(&lease)) {
                closed.reclaim();
            }
        }
        if self.cleanup_dir {
            let _ = std::fs::remove_dir_all(&self.directory);
        }
    }
}
