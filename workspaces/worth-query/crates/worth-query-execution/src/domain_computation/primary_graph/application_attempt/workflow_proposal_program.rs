use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding},
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::effect_program::{admit_platform_effects, PlatformEffectDemand};
use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationObservedFact,
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::workflow::{
    definition::{reconstruct_compiled_definition, WorkflowDefinitionCompilationPosture},
    instance::{
        admit_workflow_transition, select_proposal_replay_transition, select_proposal_transition,
    },
    proposal::{
        derive_workflow_proposal, derive_workflow_proposal_context_identity,
        observe_workflow_proposal, visit_workflow_proposal_facts,
    },
};

mod preparation;
mod publication;

pub use preparation::{
    WorkflowProposalBindingDenial, WorkflowProposalPreparationDenial,
    WorthQueryWorkflowProposalAdapter,
};
pub use publication::{
    PerformedWorkflowProposal, PreparedWorkflowProposal, PublishedWorkflowProposalRef,
    WorkflowProposalOutcome,
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
    pub(in crate::domain_computation::primary_graph) fn materialize_workflow_proposal<
        Spec,
        Program,
    >(
        mut self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        instance: super::PublishedWorkflowInstanceRef,
        input_identity: [u8; 32],
        source_identity: Option<[u8; 32]>,
    ) -> Result<
        PreparedWorkflowProposal<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if instance.branch() != self.lease.product().product_branch() {
            return Err(denial("workflow proposal belongs to another branch"));
        }
        if !installed.operation_binding_matches::<Operation>() {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
                Operation::IDENTIFIER,
            ));
        }
        let input_type = <Operation::InputBinding as ApplicationStructuredValueBinding>::IDENTITY;
        let layout = self.lease.layout.workflow().clone();
        let published = super::PublishedWorkflowDefinitionRef::retained(
            instance.branch(),
            instance.definition_entity_id(),
            instance.definition_content_identity().clone(),
        );
        let (compiled, mut facts) = self.lease.handle().with_runtime(|runtime| {
            reconstruct_compiled_definition(
                runtime,
                self.lease.snapshot(),
                &layout,
                &published,
                instance.program_revision(),
                Spec::IDENTITY.as_str(),
                usize::from(installed.resources().maximum_definition_nodes()),
                usize::from(installed.resources().maximum_definition_connections()),
                WorkflowDefinitionCompilationPosture::Retained,
            )
        })?;
        let subject = self.admission.scope_entity_id();
        let mut observed = self.lease.handle().with_runtime(|runtime| {
            super::workflow_instance_observation::observe_workflow_instance(
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
        let Some(live_membership) = observed.live_membership else {
            return Err(denial("workflow proposal instance is already settled"));
        };
        let mut settled = observed
            .transitions
            .iter()
            .map(|transition| transition.settlement)
            .collect::<Vec<_>>();
        let selection = select_proposal_transition(
            &compiled,
            instance.entity_id(),
            &mut settled,
            Operation::IDENTIFIER,
            input_type.as_str(),
        );
        let (selected, mut replay_facts) = match selection {
            Ok(selected) => (selected, Vec::new()),
            Err(error)
                if error.kind()
                    == WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported =>
            {
                self.recover_proposal_replay(
                    &layout,
                    &compiled,
                    &instance,
                    &observed,
                    Operation::IDENTIFIER,
                    input_type.as_str(),
                    input_identity,
                    source_identity,
                )?
            }
            Err(error) => return Err(error),
        };
        let proposal = derive_workflow_proposal(
            selected.identity(),
            Operation::IDENTIFIER,
            input_type.as_str(),
            input_identity,
            source_identity,
            selected.node_path(),
        );
        let replay = !replay_facts.is_empty();
        facts.append(&mut replay_facts);
        if replay {
            let transition_count = observed.transitions.len();
            let Some(WorthQueryApplicationObservedFact::WorkflowTransitionCapacity {
                maximum_transitions,
                ..
            }) = observed.facts.iter_mut().find(|fact| {
                matches!(
                    fact,
                    WorthQueryApplicationObservedFact::WorkflowTransitionCapacity { .. }
                )
            })
            else {
                return Err(denial(
                    "workflow proposal transition capacity fact is unavailable",
                ));
            };
            *maximum_transitions = transition_count;
        }
        facts.extend(observed.facts);
        if self.facts.len().saturating_add(facts.len())
            > self
                .admission
                .allowed_graph_contract()
                .decision_fact_budget()
        {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                Operation::IDENTIFIER,
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
            false,
        );
        let mut demand = PlatformEffectDemand::default();
        visit_workflow_proposal_facts(&layout, &admitted, &proposal, |effect| {
            demand.observe(&effect)
        })?;
        let reservation = admit_platform_effects(admitted.read_set(), demand)?;
        let mut effects = Vec::new();
        visit_workflow_proposal_facts(&layout, &admitted, &proposal, |effect| {
            effects.push(effect);
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        })?;
        let validator_work_admission = reservation.materialize(&effects)?;
        Ok(PreparedWorkflowProposal {
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
                output_currentness_facts: None,
            },
            program_revision: compiled.program_revision().clone(),
            transition_identity_locator: layout.transition.identity.clone(),
            proposal_identity_locator: layout.proposal.identity.clone(),
            proposal_node_path_locator: layout.proposal.node_path.clone(),
            proposal_context_identity: derive_workflow_proposal_context_identity(
                instance.entity_id(),
            ),
            instance,
            operation: Operation::IDENTIFIER.to_owned(),
            input_type: input_type.as_str().to_owned(),
            input_identity,
            source_identity,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn recover_proposal_replay(
        &self,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        compiled: &crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
        instance: &super::PublishedWorkflowInstanceRef,
        observed: &super::workflow_instance_observation::ObservedWorkflowInstance,
        operation: &str,
        input_type: &str,
        input_identity: [u8; 32],
        source_identity: Option<[u8; 32]>,
    ) -> Result<
        (
            crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowTransition,
            Vec<WorthQueryApplicationObservedFact>,
        ),
        WorthQueryApplicationAttemptDenial,
    >{
        let mut match_found = None;
        let mut latest_occurrence = None;
        for transition in &observed.transitions {
            let Ok(selected) = select_proposal_replay_transition(
                compiled,
                instance.entity_id(),
                transition.settlement,
                operation,
                input_type,
            ) else {
                continue;
            };
            let proposal = derive_workflow_proposal(
                selected.identity(),
                operation,
                input_type,
                input_identity,
                source_identity,
                selected.node_path(),
            );
            let proposal_facts = self.lease.handle().with_runtime(|runtime| {
                observe_workflow_proposal(
                    runtime,
                    self.lease.snapshot(),
                    layout,
                    transition.entity,
                    proposal.identity(),
                )
            });
            if let Ok(proposal_facts) = proposal_facts {
                let replace = latest_occurrence
                    .map(|prior| selected.occurrence() > prior)
                    .unwrap_or(true);
                if replace {
                    latest_occurrence = Some(selected.occurrence());
                    match_found = Some((selected, proposal_facts));
                }
            }
        }
        let Some((selected, proposal_facts)) = match_found else {
            return Err(denial(
                "workflow proposal replay does not match retained facts",
            ));
        };
        Ok((selected, proposal_facts))
    }
}

fn denial(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
