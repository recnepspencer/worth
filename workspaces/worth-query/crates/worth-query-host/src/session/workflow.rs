//! Workflow vocabulary borrowed from this session's own runtime.
use super::WorthQueryHostSession;
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::application_installation::WorthQueryWorkflowVocabulary;

impl<Schema: ApplicationSchema + 'static, Program> WorthQueryHostSession<'_, Schema, Program> {
    /// Look up the workflow vocabulary for a session request or pinned snapshot.
    pub fn workflow_vocabulary<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        &self,
    ) -> Option<WorthQueryWorkflowVocabulary<'_, Schema, Spec>> {
        self.home.runtime().workflow_vocabulary()
    }
}
