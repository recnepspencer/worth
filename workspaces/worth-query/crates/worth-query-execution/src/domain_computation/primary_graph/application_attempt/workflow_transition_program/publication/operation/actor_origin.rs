//! Original native occurrence locators, never current authorization.
use super::{ApplicationProgramRevision, RequiredWorkflowOperation};
use crate::domain_computation::primary_graph::application_attempt::{
    workflow_transition_program::actor_context::OriginalActorSources, PublishedWorkflowInstanceRef,
};
impl RequiredWorkflowOperation {
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn original_sources(
        &self,
    ) -> Option<&OriginalActorSources> {
        self.original_sources.as_ref()
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn original_runtime(
        &self,
    ) -> u64 {
        self.original_runtime
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn original_instance(
        &self,
    ) -> &PublishedWorkflowInstanceRef {
        &self.original_instance
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn original_program_revision(
        &self,
    ) -> &ApplicationProgramRevision {
        &self.original_program_revision
    }
}
