use super::*;

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
    pub(super) fn materialize_terminal_transition(
        mut self,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        compiled: crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
        instance: super::super::PublishedWorkflowInstanceRef,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowTransition,
        live_membership: worth_relational::facade::identity::RelationId,
        retire_live_membership: bool,
        facts: Vec<super::super::WorthQueryApplicationObservedFact>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        if self.facts.len().saturating_add(facts.len())
            > self
                .admission
                .allowed_graph_contract()
                .decision_fact_budget()
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                self.admission.operation(),
            ));
        }
        self.facts.extend(facts);
        let subject = self.admission.scope_entity_id();
        let admitted = admit_workflow_transition(
            self,
            selected,
            instance.entity_id(),
            subject,
            live_membership,
            retire_live_membership,
        );
        let transition_identity = admitted.identity().to_owned();
        let transition_identity_bytes = *admitted.identity_bytes();
        let transition_instance = admitted.instance();
        let node_path = admitted.node_path().to_owned();
        let mut demand = PlatformEffectDemand::default();
        visit_terminal_transition_facts(&layout, &admitted, |effect| demand.observe(&effect))?;
        let reservation = admit_platform_effects(admitted.read_set(), demand)?;
        let mut effects = Vec::new();
        visit_terminal_transition_facts(&layout, &admitted, |effect| {
            effects.push(effect);
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        })?;
        let validator_work_admission = reservation.materialize(&effects)?;
        let read_set = admitted.into_read_set();
        let program = WorthQueryApplicationEffectProgram {
            read_set,
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling: 0,
            conditional_definition: None,
            platform_mutation: true,
            validator_work_admission,
            output_correspondence: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: None,
        };
        Ok(PreparedWorkflowAdvance::Transition {
            program,
            program_revision: compiled.program_revision().clone(),
            transition_identity,
            transition_identity_bytes,
            transition_identity_locator: layout.transition.identity.clone(),
            assessment_identity_locator: layout.assessment_evidence.identity.clone(),
            instance: transition_instance,
            node_path,
            assessment: None,
            supporting_identity: None,
            operation_receipt_identity: None,
            progress_update: None,
            terminal: true,
            approval: None,
            approval_identity: None,
            approval_authentication: None,
            replays: Default::default(),
        })
    }
}
