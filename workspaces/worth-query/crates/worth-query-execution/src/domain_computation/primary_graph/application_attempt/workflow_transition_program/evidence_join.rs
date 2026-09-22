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
        let required = compiled
            .required_assessments(selected.node())
            .collect::<Vec<_>>();
        let distinct = required
            .iter()
            .map(|node| node.entity())
            .collect::<std::collections::BTreeSet<_>>();
        if required.len() < 2 || distinct.len() != required.len() {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "evidence join required inventory is incomplete or duplicated",
            ));
        }
        let mut completed = 0usize;
        let mut passing = 0usize;
        let mut evidence_entities = Vec::with_capacity(required.len());
        for node in &required {
            let crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowNodeKind::Assessment {
            query,
            parameter_type,
            result_type,
            binding,
            ..
        } = node.kind() else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "evidence join requirement is not an assessment node",
            ));
        };
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
            if evidence.query != *query
                || evidence.parameter_type != *parameter_type
                || evidence.result_type != *result_type
                || evidence.binding != *binding
            {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                    "assessment evidence contract differs from its authored requirement",
                ));
            }
            completed = completed.saturating_add(1);
            passing = passing.saturating_add(usize::from(evidence.passing));
            evidence_entities.push(evidence.entity);
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
        let maximum_facts = self
            .admission
            .allowed_graph_contract()
            .decision_fact_budget()
            .saturating_sub(self.facts.len());
        let evidence_currentness = self.lease.handle().with_runtime(|runtime| {
            let mut currentness = Vec::new();
            for evidence in evidence_entities {
                let remaining = maximum_facts.saturating_sub(facts.len());
                currentness.extend(
                    super::super::workflow_instance_observation::observe_evidence_dependencies(
                        runtime,
                        self.lease.snapshot(),
                        layout,
                        evidence,
                        remaining,
                        &mut facts,
                    )?,
                );
            }
            Ok::<_, WorthQueryApplicationAttemptDenial>(currentness)
        })?;
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
            platform_mutation: true,
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
            approval: None,
            approval_identity: None,
            replays: Default::default(),
        })
    }
}
