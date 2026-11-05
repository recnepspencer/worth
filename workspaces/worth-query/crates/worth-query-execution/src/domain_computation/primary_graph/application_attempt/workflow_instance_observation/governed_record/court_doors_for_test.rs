//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::workflow_instance_program_publication::PublishedWorkflowInstanceRef;

impl WorkflowRecord {
    #[doc(hidden)]
    pub fn original_start_revision_for_test(
        &self,
    ) -> &worth_query_declaration::facade::application_program::ApplicationProgramRevision {
        &self.original_start_revision
    }
}

impl WorkflowRecord {
    #[doc(hidden)]
    pub fn reference_for_test(&self) -> &Option<PublishedWorkflowInstanceRef> {
        &self.reference
    }
}

impl WorkflowRecord {
    #[doc(hidden)]
    pub fn subject_for_test(&self) -> &EntityId {
        &self.subject
    }
}
