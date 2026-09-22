use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::*;

#[path = "approval_decision/identity.rs"]
mod identity;
mod inputs;
#[path = "approval_decision/projection.rs"]
mod projection;
#[path = "approval_decision/validation.rs"]
mod validation;
use validation::*;

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
    pub(in crate::domain_computation::primary_graph) fn materialize_workflow_approval<
        Capability,
        Spec,
        Program,
    >(
        mut self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        instance: super::super::PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &super::super::PublishedWorkflowProposalRef,
        decision: WorkflowApprovalDecision,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        validate_request_binding::<Schema, Capability, Operation, _, _, _, _>(
            &self, installed, &instance, required, proposal,
        )?;
        let layout = self.lease.layout.workflow().clone();
        let published = super::super::PublishedWorkflowDefinitionRef::retained(
            instance.branch(),
            instance.definition_entity_id(),
            instance.definition_content_identity().clone(),
        );
        let (compiled, mut facts) = crate::domain_computation::primary_graph::workflow::definition::reconstruct_compiled_definition(
                self.lease.handle(),
                self.lease.snapshot(),
                &layout,
                &published,
                instance.program_revision(),
                Spec::IDENTITY.as_str(),
                installed.support_identity_bytes(),
                usize::from(installed.resources().maximum_definition_nodes()),
                usize::from(installed.resources().maximum_definition_connections()),
                crate::domain_computation::primary_graph::workflow::definition::WorkflowDefinitionCompilationPosture::Retained,
        )?;
        let subject = self.admission.scope_entity_id();
        let maximum_transitions = usize::try_from(
            installed
                .resources()
                .maximum_retained_transitions_per_instance(),
        )
        .unwrap_or(usize::MAX);
        let mut observed = self.lease.handle().with_runtime(|runtime| {
            super::super::workflow_instance_observation::observe_workflow_instance(
                self.lease.handle(),
                runtime,
                self.lease.snapshot(),
                &layout,
                &instance,
                subject,
                compiled.lineage(),
                &compiled,
                maximum_transitions,
            )
        })?;
        let replays = publication::PreparedWorkflowTransitionReplays::retained(std::mem::take(
            &mut observed.replays,
        ));
        let approval_node = validate_requirement_definition(&compiled, required)?;
        let maximum_input_facts = self
            .admission
            .allowed_graph_contract()
            .decision_fact_budget()
            .saturating_sub(
                self.facts
                    .len()
                    .saturating_add(facts.len())
                    .saturating_add(observed.facts.len()),
            );
        let inputs = inputs::observe(
            &compiled,
            &layout,
            &instance,
            observed.progress_basis.progress(),
            approval_node,
            proposal,
            self.lease.handle(),
            self.lease.snapshot(),
            maximum_input_facts,
        )?;
        facts.extend(inputs.facts);
        let meaning = identity::derive(
            &compiled,
            required,
            proposal,
            decision,
            &self.admission,
            &inputs.evidence,
        )?;
        let approval_projection = projection::prepare(&layout, &meaning)?;
        let latest = observed
            .progress_basis
            .progress()
            .latest_transition(approval_node);
        let settled = if latest
            .is_some_and(|transition| transition.settlement().occurrence() > required.occurrence())
        {
            self.lease.handle().with_runtime(|runtime| {
                observed.ensure_history(
                    self.lease.handle(),
                    runtime,
                    self.lease.snapshot(),
                    &layout,
                    instance.entity_id(),
                    maximum_transitions,
                )
            })?;
            observed.transitions.iter().find(|transition| {
                transition.settlement.node() == approval_node
                    && transition.settlement.occurrence() == required.occurrence()
            }).map(|transition| {
                crate::domain_computation::primary_graph::workflow::instance::WorkflowTransitionLocator::new(
                    transition.entity, transition.settlement,
                )
            })
        } else {
            latest
                .filter(|transition| transition.settlement().occurrence() == required.occurrence())
        };
        if let Some(settled) = settled {
            let mut settlement_facts = Vec::new();
            self.lease.handle().with_runtime(|runtime| {
                super::super::workflow_instance_observation::observe_retained_transition(
                    runtime,
                    self.lease.snapshot(),
                    &layout,
                    settled,
                    &mut settlement_facts,
                )
            })?;
            facts.append(&mut settlement_facts);
            let selected = select_settled_replay_transition(
                &compiled,
                instance.entity_id(),
                settled.settlement(),
            )?;
            if selected.node_path() != required.node_path()
                || selected.identity() != required.transition_identity()
                || settled.settlement().occurrence() != required.occurrence()
            {
                return Err(affinity(
                    "approval requirement does not match its settled transition",
                ));
            }
            facts.append(&mut observed.facts);
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
            return Ok(PreparedWorkflowAdvance::ReplayOnly {
                read_set: self,
                transition_identity_locator: layout.transition.identity.clone(),
                assessment_identity_locator: layout.assessment_evidence.identity.clone(),
                instance: instance.entity_id(),
                approval: Some(approval_projection.clone()),
                approval_identity: Some(meaning.identity),
                replays: replays.with_probe_identity(*selected.identity_bytes()),
                denial: denial(
                    WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAlreadySettled,
                    required.node_path(),
                ),
            });
        }
        let Some(live_membership) = observed.live_membership else {
            return Ok(PreparedWorkflowAdvance::ReplayOnly {
                read_set: self,
                transition_identity_locator: layout.transition.identity.clone(),
                assessment_identity_locator: layout.assessment_evidence.identity.clone(),
                instance: instance.entity_id(),
                approval: Some(approval_projection.clone()),
                approval_identity: Some(meaning.identity),
                replays,
                denial: denial(
                    WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAlreadySettled,
                    required.node_path(),
                ),
            });
        };
        let selected = match select_current_transition(
            &compiled,
            instance.entity_id(),
            &observed.progress_basis,
        ) {
            Ok(selected) => selected,
            Err(denial)
                if denial.kind()
                    == WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported =>
            {
                return Ok(PreparedWorkflowAdvance::ReplayOnly {
                    read_set: self,
                    transition_identity_locator: layout.transition.identity.clone(),
                    assessment_identity_locator: layout.assessment_evidence.identity.clone(),
                    instance: instance.entity_id(),
                    approval: Some(approval_projection.clone()),
                    approval_identity: Some(meaning.identity),
                    replays,
                    denial,
                });
            }
            Err(denial) => return Err(denial),
        };
        let SelectedWorkflowTransitionKind::Approval(approval) = selected.kind().clone() else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
                selected.node_path(),
            ));
        };
        let expected =
            RequiredWorkflowApproval::from_selected(instance.entity_id(), &selected, approval);
        if &expected != required {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "approval requirement is stale or belongs to another transition",
            ));
        }
        let replay_probe_identity = *selected.identity_bytes();
        facts.append(&mut observed.facts);
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
        super::assessment::bind_currentness_facts(
            &mut self,
            &inputs.currentness,
            required.node_path(),
        )?;
        let evidence_currentness = std::sync::Arc::from(inputs.currentness);
        let admitted = admit_workflow_transition(
            self,
            selected,
            instance.entity_id(),
            subject,
            live_membership,
            false,
        );
        materialize_decision(
            &layout,
            compiled.program_revision().clone(),
            admitted,
            meaning,
            evidence_currentness,
            approval_projection,
        )
        .map(|prepared| prepared.with_replays(replays.with_probe_identity(replay_probe_identity)))
    }
}

fn materialize_decision<Schema, Operation, Input, Scope>(
    layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    program_revision: worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    admitted: crate::domain_computation::primary_graph::workflow::instance::AdmittedWorkflowTransition<Schema, Operation, Input, Scope>,
    meaning: crate::domain_computation::primary_graph::workflow::WorkflowApprovalMeaning,
    evidence_currentness: std::sync::Arc<
        [crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact],
    >,
    approval_projection: publication::PreparedWorkflowApprovalProjection,
) -> Result<
    PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
    WorthQueryApplicationAttemptDenial,
>
where
    Schema: ApplicationSchema,
{
    let transition_identity = admitted.identity().to_owned();
    let transition_identity_bytes = *admitted.identity_bytes();
    let instance = admitted.instance();
    let node_path = admitted.node_path().to_owned();
    let mut demand = PlatformEffectDemand::default();
    crate::domain_computation::primary_graph::workflow::visit_workflow_approval_facts(
        layout,
        &admitted,
        &meaning,
        |effect| demand.observe(&effect),
    )?;
    let reservation = admit_platform_effects(admitted.read_set(), demand)?;
    let mut effects = Vec::new();
    crate::domain_computation::primary_graph::workflow::visit_workflow_approval_facts(
        layout,
        &admitted,
        &meaning,
        |effect| {
            effects.push(effect);
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        },
    )?;
    let validator_work_admission = reservation.materialize(&effects)?;
    let progress_update = admitted.prepare_progress_update(meaning.decision.outcome(), None)?;
    Ok(PreparedWorkflowAdvance::Transition {
        program: WorthQueryApplicationEffectProgram {
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
            output_currentness_facts: Some(evidence_currentness),
        },
        program_revision,
        transition_identity,
        transition_identity_bytes,
        transition_identity_locator: layout.transition.identity.clone(),
        assessment_identity_locator: layout.assessment_evidence.identity.clone(),
        instance,
        node_path,
        assessment: None,
        supporting_identity: None,
        operation_receipt_identity: None,
        progress_update: Some(progress_update),
        approval: Some(approval_projection),
        approval_identity: Some(meaning.identity),
        replays: Default::default(),
    })
}
