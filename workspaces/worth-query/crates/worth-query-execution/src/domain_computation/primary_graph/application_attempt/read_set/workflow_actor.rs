//! The admitted principal, as a workflow record names its actor.
use super::WorthQueryCompleteApplicationReadSet;

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>
{
    pub(in crate::domain_computation::primary_graph) fn admitted_workflow_actor(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::workflow::actor::record::WorkflowAdmittedActor{
        crate::domain_computation::primary_graph::application_attempt::workflow::actor::record::WorkflowAdmittedActor::from_admission(
            &self.admission,
        )
    }
}
