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
        let mut observed = self.lease.handle().with_runtime(|runtime| {
            super::super::workflow_instance_observation::observe_workflow_instance(
                runtime,
                self.lease.snapshot(),
                &layout,
                &instance,
                subject,
                compiled.lineage(),
                usize::try_from(
                    installed
                        .resources()
                        .maximum_retained_transitions_per_instance(),
                )
                .unwrap_or(usize::MAX),
            )
        })?;
        let replays = observed
            .transitions
            .iter()
            .map(|transition| {
                select_settled_replay_transition(
                    &compiled,
                    instance.entity_id(),
                    transition.settlement,
                )
                .map(|selected| publication::PreparedWorkflowTransitionReplay {
                    identity: selected.identity().to_owned(),
                    identity_bytes: *selected.identity_bytes(),
                    node_path: selected.node_path().to_owned(),
                    operation_receipt_identity: transition.settlement.operation_receipt_identity(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        let approval_node = validate_requirement_definition(&compiled, required)?;
        let evidence_currentness = validate_inputs::<Schema>(
            &compiled,
            &layout,
            &instance,
            &observed.transitions,
            approval_node,
            proposal,
            self.lease.handle(),
            self.lease.snapshot(),
            &mut facts,
            self.admission
                .allowed_graph_contract()
                .decision_fact_budget()
                .saturating_sub(self.facts.len())
                .saturating_sub(observed.facts.len()),
        )?;
        let meaning = identity::derive(
            &compiled,
            approval_node,
            required,
            proposal,
            decision,
            &self.admission,
            &observed.transitions,
        )?;
        let approval_projection = projection::prepare(&layout, &meaning)?;
        if let Some(settled) = observed.transitions.iter().find(|transition| {
            transition.settlement.node() == approval_node
                && transition.settlement.occurrence() == required.occurrence()
        }) {
            let selected = select_settled_replay_transition(
                &compiled,
                instance.entity_id(),
                settled.settlement,
            )?;
            if selected.node_path() != required.node_path()
                || selected.identity() != required.transition_identity()
                || settled.settlement.occurrence() != required.occurrence()
            {
                return Err(affinity(
                    "approval requirement does not match its settled transition",
                ));
            }
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
        let mut settled = observed
            .transitions
            .iter()
            .map(|transition| transition.settlement)
            .collect::<Vec<_>>();
        let selected = match select_current_transition(&compiled, instance.entity_id(), &mut settled)
        {
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
            &evidence_currentness,
            required.node_path(),
        )?;
        let evidence_currentness = std::sync::Arc::from(evidence_currentness);
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
        .map(|prepared| prepared.with_replays(replays))
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_inputs<Schema>(
    compiled: &crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
    layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    instance: &super::super::PublishedWorkflowInstanceRef,
    transitions: &[super::super::workflow_instance_observation::ObservedWorkflowTransition],
    approval: worth_relational::facade::identity::EntityId,
    proposal: &super::super::PublishedWorkflowProposalRef,
    handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    facts: &mut Vec<super::super::WorthQueryApplicationObservedFact>,
    maximum_facts: usize,
) -> Result<Vec<super::super::WorthQueryApplicationObservedFact>, WorthQueryApplicationAttemptDenial>
where
    Schema: ApplicationSchema,
{
    let proposal_source = unique(compiled.approval_proposal_sources(approval), "proposal")?;
    if proposal.node_path() != proposal_source.path() {
        return Err(affinity(
            "approval proposal source differs from authored input",
        ));
    }
    let proposal_transition =
        super::super::workflow_instance_observation::latest_transition_for_node(
            transitions,
            proposal_source.entity(),
        )
        .ok_or_else(|| affinity("approval proposal transition is absent"))?;
    let mut proposal_facts = handle.with_runtime(|runtime| {
        crate::domain_computation::primary_graph::workflow::proposal::observe_workflow_proposal_identity(
            runtime,
            snapshot,
            layout,
            proposal_transition.entity,
            proposal.identity(),
        )
    })?;
    facts.append(&mut proposal_facts);
    let evidence_source = unique(compiled.approval_evidence_sources(approval), "evidence")?;
    if !matches!(
        evidence_source.kind(),
        crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowNodeKind::EvidenceJoin { .. }
    ) || !super::super::workflow_instance_observation::latest_transition_for_node(
        transitions,
        evidence_source.entity(),
    )
    .is_some_and(|transition| {
        transition.settlement.outcome()
            == worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::EvidenceSatisfied
    }) {
        return Err(affinity("approval joined evidence is absent"));
    }
    let evidence = validate_passing_evidence(compiled, evidence_source.entity(), transitions)?;
    let evidence_currentness = handle.with_runtime(|runtime| {
        let mut currentness = Vec::new();
        for evidence in evidence {
            let remaining = maximum_facts.saturating_sub(facts.len());
            currentness.extend(
                super::super::workflow_instance_observation::observe_evidence_dependencies(
                    runtime, snapshot, layout, evidence, remaining, facts,
                )?,
            );
        }
        Ok::<_, WorthQueryApplicationAttemptDenial>(currentness)
    })?;
    if proposal.definition_entity_id() != instance.definition_entity_id() {
        return Err(affinity("approval proposal definition changed"));
    }
    Ok(evidence_currentness)
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
        approval: Some(approval_projection),
        approval_identity: Some(meaning.identity),
        replays: Box::default(),
    })
}
