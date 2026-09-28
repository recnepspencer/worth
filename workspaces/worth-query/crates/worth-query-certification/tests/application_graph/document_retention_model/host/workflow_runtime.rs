//! The document-retention host's installed workflow and authentication owner.

use std::time::Duration;

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

pub struct DocumentWorkflowRuntime {
    pub(in super::super) workflow: WorthQueryWorkflowApplicationRuntime<
        DocumentRetentionSchema,
        ReviewedDocumentWorkflow,
        RetentionProgramP0,
    >,
    pub(in super::super) authentication: CertificationAuthenticationOwner,
    pub(in super::super) authentication_clock: CertificationAuthenticationClockSource,
}

impl DocumentWorkflowRuntime {
    pub fn authentication(&self) -> &CertificationAuthenticationOwner {
        &self.authentication
    }

    /// Lets a certification add vocabularies for other rostered programs.
    pub fn workflow_mut(
        &mut self,
    ) -> &mut WorthQueryWorkflowApplicationRuntime<
        DocumentRetentionSchema,
        ReviewedDocumentWorkflow,
        RetentionProgramP0,
    > {
        &mut self.workflow
    }

    pub fn advance_authentication_clock_for_test(&self, duration: Duration) {
        self.authentication_clock.advance_for_test(duration);
    }
}

impl std::ops::Deref for DocumentWorkflowRuntime {
    type Target = WorthQueryWorkflowApplicationRuntime<
        DocumentRetentionSchema,
        ReviewedDocumentWorkflow,
        RetentionProgramP0,
    >;

    fn deref(&self) -> &Self::Target {
        &self.workflow
    }
}

/// Every workflow entry names its vocabulary; this host's initial one is the
/// P0 vocabulary its workflow runtime retained at installation.
impl<'application> From<&'application DocumentWorkflowRuntime>
    for WorthQueryWorkflowVocabulary<
        'application,
        DocumentRetentionSchema,
        ReviewedDocumentWorkflow,
    >
{
    fn from(application: &'application DocumentWorkflowRuntime) -> Self {
        application.workflow.vocabulary()
    }
}
