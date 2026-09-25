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
        compiled: &crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
        instance: super::super::PublishedWorkflowInstanceRef,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowTransition,
        live_membership: worth_relational::facade::identity::RelationId,
        retire_live_membership: bool,
        mut facts: Vec<super::super::WorthQueryApplicationObservedFact>,
        assessment: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowAssessment,
        progress: &crate::domain_computation::primary_graph::workflow::instance::WorkflowInstanceProgress,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let node = compiled.node(selected.node()).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "assessment node is absent from compiled definition",
            )
        })?;
        if !matches!(
            node.kind(),
            crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowNodeKind::Assessment {
                subject,
                ..
            } if subject == &assessment.subject
        ) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "selected assessment subject differs from authored requirement",
            ));
        }
        let remaining = self
            .admission
            .allowed_graph_contract()
            .decision_fact_budget()
            .saturating_sub(self.facts.len().saturating_add(facts.len()));
        let subject = super::assessment_coverage::observe_subject(
            compiled,
            layout,
            progress,
            node,
            self.admission.scope_entity_id(),
            self.lease.handle(),
            self.lease.snapshot(),
            remaining,
        )?;
        let remaining = self
            .admission
            .allowed_graph_contract()
            .decision_fact_budget()
            .saturating_sub(
                self.facts
                    .len()
                    .saturating_add(facts.len())
                    .saturating_add(subject.facts.len()),
            );
        let applicability = self.lease.handle().with_runtime(|runtime| {
            super::assessment_applicability::observe(
                node,
                &self.lease.layout,
                runtime,
                self.lease.snapshot(),
                subject.resource,
                subject.related,
                remaining,
            )
        })?;
        facts.extend(subject.facts);
        facts.extend(applicability.facts);
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
        if !applicability.applicable {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
                "assessment requirement is not currently applicable",
            ));
        }
        let program_revision = compiled.program_revision().clone();
        let coverage = subject.coverage;
        let proposal_identity = subject.proposal_identity;
        if let Some(evidence_locator) = progress.latest_assessment_evidence(selected.node()) {
            let maximum_facts = self
                .admission
                .allowed_graph_contract()
                .decision_fact_budget()
                .saturating_sub(self.facts.len().saturating_add(facts.len()));
            let (evidence, mut retained_facts) = self.lease.handle().with_runtime(|runtime| {
                super::super::workflow_instance_observation::observe_retained_assessment_evidence(
                    runtime,
                    self.lease.snapshot(),
                    layout,
                    evidence_locator,
                    maximum_facts,
                )
            })?;
            facts.append(&mut retained_facts);
            let maximum_facts = self
                .admission
                .allowed_graph_contract()
                .decision_fact_budget()
                .saturating_sub(self.facts.len().saturating_add(facts.len()));
            let coverage_state = super::assessment_coverage::observe(
                node,
                &coverage,
                &evidence,
                layout,
                self.lease.handle(),
                self.lease.snapshot(),
                maximum_facts,
            )?;
            facts.extend(coverage_state.facts);
            if coverage_state.current {
                let currentness = coverage_state.dependencies;
                if selected.node() != progress.head() {
                    return Err(denial(
                        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAlreadySettled,
                        "compatible assessment evidence is already collected",
                    ));
                }
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
            terminal: false,
            approval: None,
            approval_identity: None,
            approval_authentication: None,
            replays: Default::default(),
        })
    }
}
