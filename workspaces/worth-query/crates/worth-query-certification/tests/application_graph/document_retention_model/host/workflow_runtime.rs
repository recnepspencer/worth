//! The document-retention host's installed workflow and authentication owner.

use std::time::Duration;
use worth_query_host::facade::declaration::application_program::ApplicationWorkflowSpec;

use worth_query_host::facade::application_installation::{
    WorthQueryWorkflowApplicationRuntime, WorthQueryWorkflowVocabulary,
};

use super::super::{
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
    pub(in super::super) workflow:
        WorthQueryWorkflowApplicationRuntime<DocumentRetentionSchema, Spec, Program>,
    pub(in super::super) authentication: CertificationAuthenticationOwner,
    pub(in super::super) authentication_clock: CertificationAuthenticationClockSource,
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
