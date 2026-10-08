use super::*;
use crate::domain_computation::primary_graph::application_attempt::check_request_live;

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
    #[allow(clippy::too_many_arguments)]
    pub(super) fn materialize_inbound_wait_transition(
        mut self,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        compiled: crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
        instance: super::super::PublishedWorkflowInstanceRef,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowTransition,
        live_membership: worth_relational::facade::identity::RelationId,
        facts: Vec<super::super::WorthQueryApplicationObservedFact>,
        inbound: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowInbound,
        allowance: crate::domain_computation::primary_graph::application_attempt::WorkflowStepAllowance,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        check_request_live(
            self.admission.publication_request(),
            self.admission.operation(),
        )?;
        self.append_completed_facts(
            facts,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )?;
        let subject = self.admission.scope_entity_id();
        let admitted = admit_workflow_transition(
            self,
            selected,
            instance.entity_id(),
            subject,
            live_membership,
            false,
            allowance,
        );
        let transition_identity = admitted.identity().to_owned();
        let transition_identity_bytes = *admitted.identity_bytes();
        let transition_instance = admitted.instance();
        let node_path = admitted.node_path().to_owned();
        let mut demand = PlatformEffectDemand::default();
        visit_workflow_transition_facts(
            layout,
            &admitted,
            worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Completed,
            WorkflowInstanceState::Ready,
            |effect| demand.observe(&effect),
        )?;
        let reservation = admit_platform_effects(admitted.read_set(), demand)?;
        let mut effects = Vec::new();
        visit_workflow_transition_facts(
            layout,
            &admitted,
            worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Completed,
            WorkflowInstanceState::Ready,
            |effect| {
                effects.push(effect);
                Ok::<(), WorthQueryApplicationAttemptDenial>(())
            },
        )?;
        reservation.materialize(&effects)?;
        let progress_update = admitted.prepare_progress_update(
            worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Completed,
            None,
        )?;
        let program = WorthQueryApplicationEffectProgram {
            read_set: admitted.into_read_set(),
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling: 0,
            conditional_definition: None,
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform,
            output_correspondence: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: None,
        };
        Ok(PreparedWorkflowAdvance::Transition {
            program,
            program_revision: *compiled.program_revision(),
            transition_identity,
            transition_identity_bytes,
            transition_identity_locator: layout.transition.identity.clone(),
            assessment_identity_locator: layout.assessment_evidence.identity.clone(),
            instance: transition_instance,
            node_path,
            terminal: false,
            assessment: None,
            supporting_identity: Some(inbound.origin_receipt_identity),
            operation_receipt_identity: None,
            progress_update: Some(progress_update),
            approval: None,
            approval_identity: None,
            approval_authentication: None,
            replays: Default::default(),
        })
    }
}
