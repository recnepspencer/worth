use crate::domain_computation::primary_graph::application_installation::{
    workflow_approval_authentication_intent, WorthQueryWorkflowApplicationRuntime,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::ApplicationSchema;

use super::*;

#[path = "approval_decision/identity.rs"]
mod identity;
mod inputs;
#[path = "approval_decision/materialization.rs"]
mod materialization;
mod operation_inputs;
#[path = "approval_decision/projection.rs"]
mod projection;
#[path = "approval_decision/validation.rs"]
mod validation;
use materialization::materialize_decision;
pub(super) use operation_inputs::observe_operation_approval_inputs;
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
        workflow: &WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>,
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
        let installed = workflow.workflow_spec();
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
                installed.resources().history_reconstruction_budget(),
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
            &self.lease.layout,
            self.admission.scope_entity_id(),
            &instance,
            observed.progress_basis.progress(),
            approval_node,
            Some(proposal),
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
        let older_settlement = latest
            .is_some_and(|transition| transition.settlement().occurrence() > required.occurrence());
        let settled = if older_settlement {
            self.lease.handle().with_runtime(|runtime| {
                observed.ensure_history(
                    self.lease.handle(),
                    runtime,
                    self.lease.snapshot(),
                    &layout,
                    instance.entity_id(),
                    maximum_transitions,
                    &compiled,
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
            let (selected_path, selected_identity, probe_identity) = if older_settlement {
                // Current back-edge counters cannot reconstruct an earlier
                // occurrence. History is charged and observed on an exact
                // basis above, including the published transition identity.
                let historical = observed
                    .transitions
                    .iter()
                    .find(|transition| transition.entity == settled.entity())
                    .ok_or_else(|| affinity("settled approval is absent from observed history"))?;
                let node = compiled
                    .node(historical.settlement.node())
                    .ok_or_else(|| affinity("settled approval node is absent"))?;
                let bytes = decode_transition_identity(&historical.identity)
                    .ok_or_else(|| affinity("settled approval identity is malformed"))?;
                (node.path().to_owned(), historical.identity.clone(), bytes)
            } else {
                let selected = select_settled_replay_transition(
                    &compiled,
                    instance.entity_id(),
                    settled.settlement(),
                    observed.progress_basis.progress().back_edge_iterations(),
                )?;
                (
                    selected.node_path().to_owned(),
                    selected.identity().to_owned(),
                    *selected.identity_bytes(),
                )
            };
            if selected_path != required.node_path()
                || selected_identity != required.transition_identity()
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
                replays: replays.with_probe_identity(probe_identity),
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
        let authentication_intent = identity::authentication_intent(
            &instance,
            required,
            proposal,
            decision,
            &inputs.evidence,
            &inputs.currentness,
        );
        let evidence_currentness = std::sync::Arc::from(inputs.currentness);
        let signing_basis = std::sync::Arc::clone(&evidence_currentness);
        let admitted = admit_workflow_transition(
            self,
            selected,
            instance.entity_id(),
            subject,
            live_membership,
            false,
        );
        let mut prepared = materialize_decision(
            &layout,
            compiled.program_revision().clone(),
            admitted,
            meaning,
            evidence_currentness,
            approval_projection,
        )?;
        let authentication = publication::PreparedWorkflowApprovalAuthentication::pending(
            workflow.authentication_owner().clone(),
            authentication_intent,
            signing_basis,
            &prepared,
        );
        if let PreparedWorkflowAdvance::Transition {
            approval_authentication,
            ..
        } = &mut prepared
        {
            *approval_authentication = Some(authentication);
        }
        Ok(prepared.with_replays(replays.with_probe_identity(replay_probe_identity)))
    }
}
