use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::effect_program::{admit_platform_effects, PlatformEffectDemand};
use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::workflow::{
    definition::{reconstruct_compiled_definition, WorkflowDefinitionCompilationPosture},
    instance::{
        admit_workflow_transition, select_current_transition, select_settled_replay_transition,
        select_terminal_transition, visit_terminal_transition_facts,
        visit_workflow_transition_facts, SelectedWorkflowTransitionKind, WorkflowInstanceState,
    },
};

mod approval;
mod approval_decision;
mod assessment;
mod assessment_materialization;
mod condition;
mod condition_materialization;
mod evidence_join;
mod operation;
mod preparation;
mod publication;

pub use preparation::{
    WorkflowTransitionBindingDenial, WorkflowTransitionPreparationDenial,
    WorthQueryWorkflowAdvanceAdapter,
};
pub use publication::{
    PerformedWorkflowApproval, PerformedWorkflowAssessmentEvidence, PerformedWorkflowTransition,
    PreparedWorkflowAdvance, PreparedWorkflowAssessment, PreparedWorkflowCondition,
    PreparedWorkflowOperation, RequiredWorkflowApproval, RequiredWorkflowAssessment,
    RequiredWorkflowCondition, RequiredWorkflowEvidence, RequiredWorkflowOperation,
    WorkflowApprovalDecision, WorkflowProgressOutcome,
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
    pub(in crate::domain_computation::primary_graph) fn materialize_workflow_advance<
        Capability,
        Spec,
        Program,
    >(
        self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        instance: super::PublishedWorkflowInstanceRef,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if instance.branch() != self.lease.product().product_branch() {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                self.admission.operation(),
            ));
        }
        if !installed.advance_binding_matches::<Capability, Operation>()
            || self.admission.installed_capability_identity()
                != Some(*installed.advance_capability_identity_bytes())
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
                self.admission.operation(),
            ));
        }
        let layout = self.lease.layout.workflow().clone();
        let published = super::PublishedWorkflowDefinitionRef::retained(
            instance.branch(),
            instance.definition_entity_id(),
            instance.definition_content_identity().clone(),
        );
        let (compiled, mut facts) = reconstruct_compiled_definition(
            self.lease.handle(),
            self.lease.snapshot(),
            &layout,
            &published,
            instance.program_revision(),
            Spec::IDENTITY.as_str(),
            installed.support_identity_bytes(),
            usize::from(installed.resources().maximum_definition_nodes()),
            usize::from(installed.resources().maximum_definition_connections()),
            WorkflowDefinitionCompilationPosture::Retained,
        )?;
        let subject = self.admission.scope_entity_id();
        let maximum_transitions = usize::try_from(
            installed
                .resources()
                .maximum_retained_transitions_per_instance(),
        )
        .unwrap_or(usize::MAX);
        let mut observed = self.lease.handle().with_runtime(|runtime| {
            super::workflow_instance_observation::observe_workflow_instance(
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
        let (live_membership, retire_live_membership) = match observed.live_membership {
            Some(membership) => (membership, true),
            None => {
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
                if observed.transitions.is_empty() {
                    return Err(denial(
                        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAlreadySettled,
                        "settled workflow instance has no exact transition",
                    ));
                }
                let (settled_index, _) = observed
                    .transitions
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, transition)| transition.settlement.occurrence())
                    .expect("settled transition inventory was checked as nonempty");
                let settled = observed.transitions.remove(settled_index);
                let selected = select_terminal_transition(
                    &compiled,
                    instance.entity_id(),
                    &observed.progress_basis,
                )?;
                if !selected.matches_settlement(settled.settlement) {
                    return Err(denial(
                        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                        "settled workflow transition does not close the compiled terminal head",
                    ));
                }
                let replay_probe_identity = *selected.identity_bytes();
                let (membership, mut settlement_facts) =
                    self.lease.handle().with_runtime(|runtime| {
                        super::workflow_instance_observation::recover_settled_live_membership(
                            runtime,
                            self.lease.snapshot(),
                            &layout,
                            settled.entity,
                            selected.identity(),
                            selected.occurrence(),
                        )
                    })?;
                observed.facts.append(&mut settlement_facts);
                facts.append(&mut observed.facts);
                let replays = publication::PreparedWorkflowTransitionReplays::retained(
                    std::mem::take(&mut observed.replays),
                );
                return self
                    .materialize_terminal_transition(
                        &layout, compiled, instance, selected, membership, false, facts,
                    )
                    .map(|prepared| {
                        prepared.with_replays(replays.with_probe_identity(replay_probe_identity))
                    });
            }
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
                let replays = publication::PreparedWorkflowTransitionReplays::retained(
                    std::mem::take(&mut observed.replays),
                );
                return Ok(PreparedWorkflowAdvance::ReplayOnly {
                    read_set: self,
                    transition_identity_locator: layout.transition.identity.clone(),
                    assessment_identity_locator: layout.assessment_evidence.identity.clone(),
                    instance: instance.entity_id(),
                    approval: None,
                    approval_identity: None,
                    replays,
                    denial,
                });
            }
            Err(denial) => return Err(denial),
        };
        if matches!(
            selected.kind(),
            SelectedWorkflowTransitionKind::Assessment(_)
                | SelectedWorkflowTransitionKind::EvidenceJoin(_)
                | SelectedWorkflowTransitionKind::Operation(_)
        ) {
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
        }
        let replay_probe_identity = *selected.identity_bytes();
        let replays = publication::PreparedWorkflowTransitionReplays::retained(std::mem::take(
            &mut observed.replays,
        ));
        facts.append(&mut observed.facts);
        let prepared = match selected.kind().clone() {
            SelectedWorkflowTransitionKind::Assessment(assessment) => self
                .materialize_assessment_requirement(
                    &layout,
                    compiled.program_revision().clone(),
                    instance,
                    selected,
                    live_membership,
                    false,
                    facts,
                    assessment,
                    &observed.transitions,
                ),
            SelectedWorkflowTransitionKind::Condition(condition) => self
                .materialize_condition_requirement(
                    &layout,
                    compiled.program_revision().clone(),
                    instance,
                    selected,
                    live_membership,
                    false,
                    facts,
                    condition,
                ),
            SelectedWorkflowTransitionKind::Approval(approval) => {
                self.materialize_approval_requirement(&layout, instance, selected, facts, approval)
            }
            SelectedWorkflowTransitionKind::EvidenceJoin(policy) => self.materialize_evidence_join(
                &layout,
                compiled,
                instance,
                selected,
                live_membership,
                facts,
                &observed.transitions,
                policy,
            ),
            SelectedWorkflowTransitionKind::Terminal => self.materialize_terminal_transition(
                &layout,
                compiled,
                instance,
                selected,
                live_membership,
                retire_live_membership,
                facts,
            ),
            SelectedWorkflowTransitionKind::Operation(operation) => self
                .materialize_operation_requirement(
                    &layout,
                    &compiled,
                    instance,
                    selected,
                    live_membership,
                    false,
                    facts,
                    &observed.transitions,
                    operation,
                ),
        }?;
        Ok(prepared.with_replays(replays.with_probe_identity(replay_probe_identity)))
    }

    #[allow(clippy::too_many_arguments)]
    fn materialize_terminal_transition(
        mut self,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        compiled: crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
        instance: super::PublishedWorkflowInstanceRef,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowTransition,
        live_membership: worth_relational::facade::identity::RelationId,
        retire_live_membership: bool,
        facts: Vec<super::WorthQueryApplicationObservedFact>,
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
            approval: None,
            approval_identity: None,
            replays: Default::default(),
        })
    }
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
