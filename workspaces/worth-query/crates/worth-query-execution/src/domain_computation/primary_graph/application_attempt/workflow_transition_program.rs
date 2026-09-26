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
        admit_workflow_transition, select_assessment_collection, select_current_transition,
        select_navigation_back_transition, select_settled_replay_transition,
        select_terminal_transition, visit_terminal_transition_facts,
        visit_workflow_transition_facts, SelectedWorkflowTransitionKind, WorkflowInstanceState,
    },
};

mod approval;
mod approval_decision;
mod assessment;
mod assessment_applicability;
mod assessment_coverage;
mod assessment_materialization;
mod condition;
mod condition_materialization;
mod evidence_join;
mod navigation;
mod operation;
pub(super) use operation::{operation_receipt_requires_recovery, receipt_identity_from_outcome};
mod preparation;
mod publication;
pub(super) use publication::transition_entity_in_receipt;
mod terminal;

pub use preparation::{
    WorkflowTransitionBindingDenial, WorkflowTransitionPreparationDenial,
    WorthQueryWorkflowAdvanceAdapter,
};
pub use publication::{
    PerformedWorkflowApproval, PerformedWorkflowAssessmentEvidence, PerformedWorkflowTransition,
    PreparedWorkflowAdvance, PreparedWorkflowAssessment, PreparedWorkflowCondition,
    PreparedWorkflowOperation, RequiredWorkflowActor, RequiredWorkflowApproval,
    RequiredWorkflowAssessment, RequiredWorkflowCondition, RequiredWorkflowEvidence,
    RequiredWorkflowOperation, WorkflowApprovalDecision, WorkflowOperationAuthority,
    WorkflowOperationAuthoritySlot, WorkflowProgressOutcome,
};

#[derive(Clone, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorkflowTransitionRequestKind {
    Advance,
    NavigateBack,
    CollectAssessment { node_path: String },
}

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
        request_kind: WorkflowTransitionRequestKind,
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
            installed.program_revision(),
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
                installed.resources().history_reconstruction_budget(),
            )
        })?;
        let (live_membership, retire_live_membership) = match observed.live_membership {
            Some(membership) => (membership, true),
            None => {
                if request_kind == WorkflowTransitionRequestKind::NavigateBack {
                    return Ok(self.navigation_replay_denial(
                        &layout,
                        instance.entity_id(),
                        std::mem::take(&mut observed.replays),
                        denial(
                            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAlreadySettled,
                            "settled workflow instance cannot navigate Back",
                        ),
                    ));
                }
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
        if request_kind == WorkflowTransitionRequestKind::NavigateBack {
            if observed.progress_basis.progress().next_occurrence() >= maximum_transitions as u64 {
                return Ok(self.navigation_replay_denial(
                    &layout,
                    instance.entity_id(),
                    std::mem::take(&mut observed.replays),
                    denial(
                        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionCapacityExceeded,
                        "workflow transition retention capacity is exhausted",
                    ),
                ));
            }
            let selected = match select_navigation_back_transition(
                &compiled,
                instance.entity_id(),
                &observed.progress_basis,
            ) {
                Ok(selected) => selected,
                Err(denial) => {
                    return Ok(self.navigation_replay_denial(
                        &layout,
                        instance.entity_id(),
                        std::mem::take(&mut observed.replays),
                        denial,
                    ));
                }
            };
            let replay_probe_identity = *selected.identity_bytes();
            let replays = publication::PreparedWorkflowTransitionReplays::retained(std::mem::take(
                &mut observed.replays,
            ))
            .for_navigation_back();
            facts.append(&mut observed.facts);
            return self
                .materialize_navigation_back(
                    &layout,
                    compiled,
                    instance,
                    selected,
                    live_membership,
                    facts,
                )
                .map(|prepared| {
                    prepared.with_replays(replays.with_probe_identity(replay_probe_identity))
                });
        }
        let selection = match request_kind {
            WorkflowTransitionRequestKind::Advance => {
                select_current_transition(&compiled, instance.entity_id(), &observed.progress_basis)
            }
            WorkflowTransitionRequestKind::CollectAssessment { ref node_path } => {
                select_assessment_collection(
                    &compiled,
                    instance.entity_id(),
                    &observed.progress_basis,
                    node_path,
                )
            }
            WorkflowTransitionRequestKind::NavigateBack => unreachable!("Back returned above"),
        };
        let selected = match selection {
            Ok(selected) => selected,
            Err(denial)
                if denial.kind()
                    == WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported
                    && matches!(request_kind, WorkflowTransitionRequestKind::Advance) =>
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
        let replay_probe_identity = *selected.identity_bytes();
        let replays = publication::PreparedWorkflowTransitionReplays::retained(std::mem::take(
            &mut observed.replays,
        ));
        let handoff_facts = matches!(
            selected.kind(),
            SelectedWorkflowTransitionKind::Operation(_)
        )
        .then(|| observed.facts[..observed.handoff_fact_count].to_vec());
        facts.append(&mut observed.facts);
        let prepared = match selected.kind().clone() {
            SelectedWorkflowTransitionKind::Assessment(assessment) => self
                .materialize_assessment_requirement(
                    &layout,
                    &compiled,
                    instance,
                    selected,
                    live_membership,
                    false,
                    facts,
                    assessment,
                    observed.progress_basis.progress(),
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
                observed.progress_basis.progress(),
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
                    handoff_facts.expect("operation handoff facts were selected"),
                    observed.progress_basis.progress(),
                    operation,
                ),
            SelectedWorkflowTransitionKind::NavigationBack => Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
                "navigation requires the explicit Back action",
            )),
        }?;
        Ok(prepared.with_replays(replays.with_probe_identity(replay_probe_identity)))
    }
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
