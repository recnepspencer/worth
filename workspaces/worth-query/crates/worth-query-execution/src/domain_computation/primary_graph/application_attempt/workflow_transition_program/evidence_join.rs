use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::ApplicationSchema;

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
    pub(super) fn materialize_evidence_join(
        mut self,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        compiled: crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
        instance: super::super::PublishedWorkflowInstanceRef,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowTransition,
        live_membership: worth_relational::facade::identity::RelationId,
        mut facts: Vec<super::super::WorthQueryApplicationObservedFact>,
        progress: &crate::domain_computation::primary_graph::workflow::instance::WorkflowInstanceProgress,
        policy: worth_query_declaration::facade::application_program::ApplicationWorkflowEvidenceJoinPolicy,
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
        let authored = compiled
            .required_assessments(selected.node())
            .collect::<Vec<_>>();
        let distinct = authored
            .iter()
            .map(|node| node.entity())
            .collect::<std::collections::BTreeSet<_>>();
        if authored.len() < 2 || distinct.len() != authored.len() {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "evidence join required inventory is incomplete or duplicated",
            ));
        }
        let mut required = Vec::new();
        let mut observed_proposals =
            super::assessment_coverage::AssessmentProposalObservations::default();
        for node in authored {
            let remaining = self
                .admission
                .allowed_graph_contract()
                .decision_fact_budget()
                .saturating_sub(self.facts.len().saturating_add(facts.len()));
            let subject = super::assessment_coverage::observe_subject_cached(
                &compiled,
                layout,
                progress,
                node,
                self.admission.scope_entity_id(),
                self.lease.handle(),
                self.lease.snapshot(),
                remaining,
                &mut observed_proposals,
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
            let observed = self.lease.handle().with_runtime(|runtime| {
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
            facts.extend(observed.facts);
            if observed.applicable {
                required.push((node, subject.coverage));
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
        if required.is_empty() {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "evidence join has no currently applicable authored requirement",
            ));
        }
        let mut completed = 0usize;
        let mut passing = 0usize;
        let mut evidence_currentness = Vec::new();
        for (node, coverage) in &required {
            let Some(evidence_locator) = progress.latest_assessment_evidence(node.entity()) else {
                continue;
            };
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
            let observed = super::assessment_coverage::observe(
                node,
                coverage,
                &evidence,
                layout,
                self.lease.handle(),
                self.lease.snapshot(),
                maximum_facts,
            )?;
            facts.extend(observed.facts);
            if !observed.current {
                continue;
            }
            completed = completed.saturating_add(1);
            passing = passing.saturating_add(usize::from(evidence.passing));
            evidence_currentness.extend(observed.dependencies);
        }
        if completed != required.len() {
            let required = RequiredWorkflowEvidence::new(
                instance.entity_id(),
                selected.node_path().to_owned(),
                required.len(),
                completed,
                passing,
            );
            return Ok(PreparedWorkflowAdvance::AwaitingEvidence {
                read_set: self,
                transition_identity_locator: layout.transition.identity.clone(),
                assessment_identity_locator: layout.assessment_evidence.identity.clone(),
                instance: instance.entity_id(),
                required,
                replays: Default::default(),
            });
        }
        let satisfied = match policy {
            worth_query_declaration::facade::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing => passing == required.len(),
            worth_query_declaration::facade::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredCompleted => true,
        };
        let outcome = if satisfied {
            worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::EvidenceSatisfied
        } else {
            worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::EvidenceFailed
        };
        self.facts.extend(facts);
        super::assessment::bind_currentness_facts(
            &mut self,
            &evidence_currentness,
            selected.node_path(),
        )?;
        let evidence_currentness = std::sync::Arc::from(evidence_currentness);
        let subject = self.admission.scope_entity_id();
        let admitted = admit_workflow_transition(
            self,
            selected,
            instance.entity_id(),
            subject,
            live_membership,
            false,
        );
        let transition_identity = admitted.identity().to_owned();
        let transition_identity_bytes = *admitted.identity_bytes();
        let transition_instance = admitted.instance();
        let node_path = admitted.node_path().to_owned();
        let mut demand = PlatformEffectDemand::default();
        crate::domain_computation::primary_graph::workflow::instance::visit_workflow_transition_facts(
        layout,
        &admitted,
        outcome,
        crate::domain_computation::primary_graph::workflow::instance::WorkflowInstanceState::Ready,
        |effect| demand.observe(&effect),
    )?;
        let reservation = admit_platform_effects(admitted.read_set(), demand)?;
        let mut effects = Vec::new();
        crate::domain_computation::primary_graph::workflow::instance::visit_workflow_transition_facts(
        layout,
        &admitted,
        outcome,
        crate::domain_computation::primary_graph::workflow::instance::WorkflowInstanceState::Ready,
        |effect| {
            effects.push(effect);
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        },
        )?;
        let validator_work_admission = reservation.materialize(&effects)?;
        let progress_update = admitted.prepare_progress_update(outcome, None)?;
        let read_set = admitted.into_read_set();
        let program = WorthQueryApplicationEffectProgram {
            read_set,
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling: 0,
            conditional_definition: None,
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform,
            validator_work_admission,
            output_correspondence: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: Some(evidence_currentness),
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
            progress_update: Some(progress_update),
            terminal: false,
            approval: None,
            approval_identity: None,
            approval_authentication: None,
            replays: Default::default(),
        })
    }
}
