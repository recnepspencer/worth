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
    pub(super) fn materialize_assessment_requirement(
        mut self,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        program_revision: worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        instance: super::super::PublishedWorkflowInstanceRef,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowTransition,
        live_membership: worth_relational::facade::identity::RelationId,
        retire_live_membership: bool,
        mut facts: Vec<super::super::WorthQueryApplicationObservedFact>,
        assessment: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowAssessment,
        transitions: &[super::super::workflow_instance_observation::ObservedWorkflowTransition],
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let proposal_fact_budget = self
            .admission
            .allowed_graph_contract()
            .decision_fact_budget()
            .saturating_sub(self.facts.len().saturating_add(facts.len()));
        let (proposal_entity, proposal_identity, proposal_facts) = self.lease.handle().with_runtime(|runtime| {
            super::super::workflow_instance_observation::observe_latest_workflow_proposal_identity(
                runtime,
                self.lease.snapshot(),
                layout,
                transitions,
                proposal_fact_budget,
            )
        })?;
        facts.extend(proposal_facts);
        let (coverage, coverage_facts) = self.lease.handle().with_runtime(|runtime| {
            crate::domain_computation::primary_graph::workflow::proposal::observe_workflow_proposal_coverage(
                runtime,
                self.lease.snapshot(),
                layout,
                proposal_entity,
                &assessment.subject,
            )
        })?;
        facts.extend(coverage_facts);
        if let Some(evidence) =
            super::super::workflow_instance_observation::latest_assessment_evidence(
                transitions,
                selected.node(),
            )
        {
            if evidence.query != assessment.query
                || evidence.parameter_type != assessment.parameter_type
                || evidence.result_type != assessment.result_type
                || evidence.binding != assessment.binding
            {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                    "retained assessment evidence contract differs from its authored requirement",
                ));
            }
            if evidence.coverage_identity == coverage.identity {
                let maximum_facts = self
                    .admission
                    .allowed_graph_contract()
                    .decision_fact_budget()
                    .saturating_sub(self.facts.len().saturating_add(facts.len()));
                let mut evidence_facts = Vec::new();
                let currentness = self.lease.handle().with_runtime(|runtime| {
                    super::super::workflow_instance_observation::observe_evidence_dependencies(
                        runtime,
                        self.lease.snapshot(),
                        layout,
                        evidence.entity,
                        maximum_facts,
                        &mut evidence_facts,
                    )
                })?;
                let current = self.lease.handle().with_runtime(|runtime| {
                    currentness
                        .iter()
                        .all(|fact| fact.remains_equal_in(runtime, self.lease.snapshot()))
                });
                if current {
                    facts.extend(evidence_facts);
                    return self.materialize_reused_assessment_transition(
                        layout,
                        program_revision,
                        instance,
                        selected,
                        live_membership,
                        retire_live_membership,
                        facts,
                        currentness,
                        coverage.subject,
                    );
                }
            }
        }
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
        let subject = coverage.subject;
        let admitted = admit_workflow_transition(
            self,
            selected,
            instance.entity_id(),
            subject,
            live_membership,
            retire_live_membership,
        );
        let required = RequiredWorkflowAssessment::from_selected(
            admitted.instance(),
            admitted.node_path().to_owned(),
            admitted.identity().to_owned(),
            admitted.occurrence(),
            assessment,
            proposal_identity,
            coverage.identity,
        );
        Ok(PreparedWorkflowAdvance::AwaitingAssessment(
            PreparedWorkflowAssessment {
                admitted,
                required,
                layout: layout.clone(),
                program_revision,
                replays: Default::default(),
            },
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn materialize_reused_assessment_transition(
        mut self,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        program_revision: worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        instance: super::super::PublishedWorkflowInstanceRef,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowTransition,
        live_membership: worth_relational::facade::identity::RelationId,
        retire_live_membership: bool,
        facts: Vec<super::super::WorthQueryApplicationObservedFact>,
        currentness: Vec<super::super::WorthQueryApplicationObservedFact>,
        subject: worth_relational::facade::identity::EntityId,
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
        assessment::bind_currentness_facts(&mut self, &currentness, selected.node_path())?;
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
        let node_path = admitted.node_path().to_owned();
        let mut demand = PlatformEffectDemand::default();
        visit_workflow_transition_facts(
            layout, &admitted,
            worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Completed,
            WorkflowInstanceState::Ready, |effect| demand.observe(&effect),
        )?;
        let reservation = admit_platform_effects(admitted.read_set(), demand)?;
        let mut effects = Vec::new();
        visit_workflow_transition_facts(
            layout, &admitted,
            worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Completed,
            WorkflowInstanceState::Ready,
            |effect| { effects.push(effect); Ok::<(), WorthQueryApplicationAttemptDenial>(()) },
        )?;
        let validator_work_admission = reservation.materialize(&effects)?;
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
            platform_mutation: true,
            validator_work_admission,
            output_correspondence: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: Some(std::sync::Arc::from(currentness)),
        };
        Ok(PreparedWorkflowAdvance::Transition {
            program,
            program_revision,
            transition_identity,
            transition_identity_bytes,
            transition_identity_locator: layout.transition.identity.clone(),
            assessment_identity_locator: layout.assessment_evidence.identity.clone(),
            instance: instance.entity_id(),
            node_path,
            assessment: None,
            supporting_identity: None,
            operation_receipt_identity: None,
            progress_update: Some(progress_update),
            approval: None,
            approval_identity: None,
            replays: Default::default(),
        })
    }
}
