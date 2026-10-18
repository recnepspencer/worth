//! Waiting for an original proposal retains observation without advancing the instance.
use super::{
    publication, ApplicationOperationMarkerIdentity, ApplicationSchema, PreparedWorkflowAdvance,
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;
use crate::domain_computation::primary_graph::{
    application_attempt::workflow_instance_observation::ObservedWorkflowInstance,
    workflow::instance::SelectedWorkflowProposal,
};
impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    pub(super) fn await_proposal(
        self,
        layout: &WorthQueryWorkflowLayout,
        instance: worth_relational::facade::identity::EntityId,
        proposal: SelectedWorkflowProposal,
        observed: &mut ObservedWorkflowInstance,
    ) -> PreparedWorkflowAdvance<Schema, Operation, Input, Scope> {
        let required = publication::RequiredWorkflowProposal::from_selected(
            instance,
            observed.progress_basis.progress().next_occurrence(),
            proposal,
        );
        // Waiting describes native truth; it neither publishes a transition
        // nor consumes a step or establishes a new admission deadline.
        PreparedWorkflowAdvance::AwaitingProposal {
            read_set: self,
            transition_identity_locator: layout.transition.identity.clone(),
            assessment_identity_locator: layout.assessment_evidence.identity.clone(),
            instance,
            required,
            replays: publication::PreparedWorkflowTransitionReplays::retained(std::mem::take(
                &mut observed.replays,
            )),
        }
    }
}
