//! The real retention host with a separate governed workflow vocabulary.
pub use super::super::super::{host, programs::RetentionProgramP0};
pub use super::*;
pub use worth_query_host::facade::application_installation::WorthQueryWorkflowApplicationRuntime;
pub type GovernedRuntime = WorthQueryWorkflowApplicationRuntime<
    DocumentRetentionSchema,
    GovernedDocumentWorkflow,
    RetentionProgramP0,
>;
pub type GovernedDirectoryHome = super::super::super::host::DocumentWorkflowDirectoryHome<
    RetentionProgramP0,
    GovernedDocumentWorkflow,
>;

pub trait GovernedWorkflowOwner {
    fn program_runtime(
        &self,
    ) -> &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema,
        RetentionProgramP0,
    >;
    fn runtime(
        &self,
    ) -> &worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        DocumentRetentionSchema,
    > {
        self.program_runtime().runtime()
    }
    fn vocabulary(
        &self,
    ) -> worth_query_host::facade::application_installation::WorthQueryWorkflowVocabulary<
        '_,
        DocumentRetentionSchema,
        GovernedDocumentWorkflow,
    >;
    fn workflow_spec(
        &self,
    ) -> &worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
        DocumentRetentionSchema,
        GovernedDocumentWorkflow,
    >;
    fn support_workflow_spec(
        &mut self,
        workflow: worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
            DocumentRetentionSchema,
            GovernedDocumentWorkflow,
        >,
    ) -> Result<
        (),
        (
            worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
                DocumentRetentionSchema,
                GovernedDocumentWorkflow,
            >,
            worth_query_host::facade::application_installation::WorthQueryWorkflowRuntimeBindingDenial,
        ),
    >;
}

impl GovernedWorkflowOwner for GovernedRuntime {
    fn program_runtime(
        &self,
    ) -> &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema,
        RetentionProgramP0,
    > {
        WorthQueryWorkflowApplicationRuntime::program_runtime(self)
    }
    fn vocabulary(
        &self,
    ) -> worth_query_host::facade::application_installation::WorthQueryWorkflowVocabulary<
        '_,
        DocumentRetentionSchema,
        GovernedDocumentWorkflow,
    > {
        WorthQueryWorkflowApplicationRuntime::vocabulary(self)
    }
    fn workflow_spec(
        &self,
    ) -> &worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
        DocumentRetentionSchema,
        GovernedDocumentWorkflow,
    > {
        WorthQueryWorkflowApplicationRuntime::workflow_spec(self)
    }
    fn support_workflow_spec(
        &mut self,
        workflow: worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
            DocumentRetentionSchema,
            GovernedDocumentWorkflow,
        >,
    ) -> Result<
        (),
        (
            worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
                DocumentRetentionSchema,
                GovernedDocumentWorkflow,
            >,
            worth_query_host::facade::application_installation::WorthQueryWorkflowRuntimeBindingDenial,
        ),
    > {
        WorthQueryWorkflowApplicationRuntime::support_workflow_spec(self, workflow)
    }
}

impl GovernedWorkflowOwner for GovernedDirectoryHome {
    fn program_runtime(
        &self,
    ) -> &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema,
        RetentionProgramP0,
    > {
        self.home().runtime()
    }
    fn vocabulary(
        &self,
    ) -> worth_query_host::facade::application_installation::WorthQueryWorkflowVocabulary<
        '_,
        DocumentRetentionSchema,
        GovernedDocumentWorkflow,
    > {
        self.home().workflow_vocabulary().expect("workflow registered")
    }
    fn workflow_spec(
        &self,
    ) -> &worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
        DocumentRetentionSchema,
        GovernedDocumentWorkflow,
    > {
        self.home().retained_workflow_spec().expect("workflow registered")
    }
    fn support_workflow_spec(
        &mut self,
        workflow: worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
            DocumentRetentionSchema,
            GovernedDocumentWorkflow,
        >,
    ) -> Result<
        (),
        (
            worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
                DocumentRetentionSchema,
                GovernedDocumentWorkflow,
            >,
            worth_query_host::facade::application_installation::WorthQueryWorkflowRuntimeBindingDenial,
        ),
    > {
        self.home_mut().support_workflow_spec(workflow)
    }
}

pub fn open_directory(directory: std::path::PathBuf) -> GovernedDirectoryHome {
    let queries = worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile::new(
        std::num::NonZeroUsize::new(128).expect("concurrent graph work is non-zero"),
    );
    host::open_workflow_directory_for::<RetentionProgramP0, GovernedDocumentWorkflow>(
        directory,
        super::super::super::programs::validated_first_program(),
        host::default_first_program_roster(),
        host::with_query_resources(
            queries,
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        super::super::runtime::workflow_resources(),
    )
}
pub fn install() -> GovernedRuntime {
    // Native request owners qualify exact fork reads and their catalog backing;
    // this profile selects simultaneous graph admission, not a work invoice.
    let queries = worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile::new(
        std::num::NonZeroUsize::new(128).expect("concurrent graph work is non-zero"),
    );
    retain(host::publish_on_first_program_with_limits(
        host::with_query_resources(queries, worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
    ))
}
pub fn retain(
    application: super::super::super::host::DocumentRetentionRuntime<RetentionProgramP0>,
) -> GovernedRuntime {
    retain_with_auth(application).0
}
pub fn retain_with_auth(
    application: super::super::super::host::DocumentRetentionRuntime<RetentionProgramP0>,
) -> (
    GovernedRuntime,
    super::super::CertificationAuthenticationOwner,
) {
    let (authentication, _clock) = super::super::install_certification_authentication_with_clock(
        application.runtime().installed_schema(),
    );
    retain_with_signing_owner(application, authentication)
}
pub fn retain_with_signing_owner(
    application: super::super::super::host::DocumentRetentionRuntime<RetentionProgramP0>,
    authentication: super::super::CertificationAuthenticationOwner,
) -> (
    GovernedRuntime,
    super::super::CertificationAuthenticationOwner,
) {
    let workflow = super::super::runtime::install_workflow_spec_for::<
        RetentionProgramP0,
        GovernedDocumentWorkflow,
    >(
        application.runtime().installed_schema(),
        application.installed_program(),
        super::super::runtime::workflow_resources(),
    );
    let runtime = application
        .retain_workflow_spec(workflow, authentication.signing_owner())
        .expect("real governed vocabulary installs");
    (runtime, authentication)
}
