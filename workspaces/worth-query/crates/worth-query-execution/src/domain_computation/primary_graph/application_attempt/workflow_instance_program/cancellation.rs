//! An explicit cancellation ends a live instance where it stands. It is not
//! rollback: every effect the instance performed remains, and the outcome
//! reports each one. A cancellation prepared before a step settles goes stale,
//! and a step admitted before the cancellation commits goes stale in turn.

mod publication;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::super::effect_program::{admit_platform_effects, PlatformEffectDemand};
use super::super::workflow_instance_observation::{
    observe_ended_history, observe_workflow_instance,
};
use super::super::{
    observe_field_value, PublishedWorkflowDefinitionRef, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationObservedFact, WorthQueryApplicationRealizedEffect,
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};
use super::{intent_identity, performed, PublishedWorkflowInstanceRef};
use crate::domain_computation::primary_graph::workflow::{
    definition::{reconstruct_compiled_definition, WorkflowDefinitionCompilationPosture},
    instance::WorkflowInstanceState,
};
pub use publication::{
    PerformedWorkflowInstanceCancellation, PreparedWorkflowInstanceCancellation,
    WorkflowInstanceCancellationOutcome,
};

const HISTORY: WorthQueryApplicationAttemptDenialKind =
    WorthQueryApplicationAttemptDenialKind::WorkflowInstanceHistoryUnavailable;

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
    pub(in crate::domain_computation::primary_graph) fn materialize_workflow_instance_cancellation<
        Capability,
        Spec,
        Program,
    >(
        mut self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        instance: PublishedWorkflowInstanceRef,
        cancel_key_identity: [u8; 32],
    ) -> Result<
        PreparedWorkflowInstanceCancellation<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let branch = self.lease.product().product_branch();
        // The start capability authorizes ending what it started, on the
        // branch whose copy the request ends.
        self.authorize_instance_start::<Capability, Spec, Program>(installed, branch)?;
        if instance.branch() != branch {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch,
                self.admission.operation(),
            ));
        }
        let layout = self.lease.layout.workflow().clone();
        let (mut compiled, mut facts) = reconstruct_compiled_definition(
            self.lease.handle(),
            self.lease.snapshot(),
            &layout,
            &PublishedWorkflowDefinitionRef::retained(
                branch,
                instance.definition_entity_id(),
                instance.definition_content_identity().clone(),
            ),
            installed.program_revision(),
            Spec::IDENTITY.as_str(),
            installed.support_identity_bytes(),
            usize::from(installed.resources().maximum_definition_nodes()),
            usize::from(installed.resources().maximum_definition_connections()),
            WorkflowDefinitionCompilationPosture::Retained,
        )?;
        let (identity, intent_identity) = intent_identity::cancellation_identity(
            instance.entity_id(),
            branch.occurrence_ordinal(),
            cancel_key_identity,
        )
        .map_err(|()| {
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowInstanceIntentIdentityUnavailable,
                self.admission.operation(),
            )
        })?;
        let maximum_transitions = usize::try_from(
            installed
                .resources()
                .maximum_retained_transitions_per_instance(),
        )
        .unwrap_or(usize::MAX);
        let budget = installed.resources().history_reconstruction_budget();
        let subject = self.admission.scope_entity_id();
        let entity = instance.entity_id();
        let operation = self.admission.operation();
        let handle = self.lease.handle();
        let snapshot = self.lease.snapshot();
        let (effects, performed) = handle.with_runtime(|runtime| {
            let observed = observe_workflow_instance(
                handle,
                runtime,
                snapshot,
                &layout,
                &instance,
                subject,
                compiled.lineage(),
                &mut compiled,
                maximum_transitions,
                budget,
            );
            let (transitions, effects) = match observed {
                // An instance this request already cancelled emits nothing.
                // Its false readiness fact never becomes true again, so the
                // program resolves the exact replay; it never commits empty.
                // Any other request for a cancelled instance is refused.
                Err(ended)
                    if ended.kind()
                        == WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled =>
                {
                    let recorded = observe_field_value(
                        runtime,
                        snapshot,
                        entity,
                        layout.instance.entity_kind,
                        &layout.instance.cancellation_identity,
                    );
                    if recorded != Some(AspectValue::String(InternedString::Raw(identity.clone())))
                    {
                        return Err(ended);
                    }
                    facts.push(WorthQueryApplicationObservedFact::Field {
                        entity_id: entity,
                        kind: layout.instance.entity_kind,
                        locator: layout.instance.state.clone(),
                        value: AspectValue::UInt64(WorkflowInstanceState::Ready.persisted_tag()),
                    });
                    let (transitions, history_facts) = observe_ended_history(
                        runtime,
                        snapshot,
                        &layout,
                        entity,
                        maximum_transitions,
                        &compiled,
                        budget,
                    )?;
                    facts.extend(history_facts);
                    (transitions, Vec::new())
                }
                Err(denial) => return Err(denial),
                Ok(mut observed) => {
                    let Some(live_membership) = observed.live_membership else {
                        return Err(WorthQueryApplicationAttemptDenial::new(
                            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCompleted,
                            operation,
                        ));
                    };
                    observed.ensure_history(
                        handle,
                        runtime,
                        snapshot,
                        &layout,
                        entity,
                        maximum_transitions,
                        &compiled,
                    )?;
                    facts.append(&mut observed.facts);
                    // A live instance never recorded a cancellation; the
                    // write below replaces that absence.
                    facts.push(WorthQueryApplicationObservedFact::AbsentField {
                        entity_id: entity,
                        kind: layout.instance.entity_kind,
                        locator: layout.instance.cancellation_identity.clone(),
                    });
                    let effects = vec![
                        WorthQueryApplicationRealizedEffect::UpdateEntity {
                            entity: "workflow-instance".to_owned(),
                            entity_id: entity,
                            fields: std::collections::BTreeMap::from([
                                (
                                    layout.instance.state.clone(),
                                    AspectValue::UInt64(
                                        WorkflowInstanceState::Cancelled.persisted_tag(),
                                    ),
                                ),
                                (
                                    layout.instance.cancellation_identity.clone(),
                                    AspectValue::String(InternedString::Raw(identity.clone())),
                                ),
                            ]),
                        },
                        WorthQueryApplicationRealizedEffect::DeleteRelation {
                            relation_id: live_membership,
                        },
                    ];
                    (observed.transitions, effects)
                }
            };
            let mut performed = performed::own_effects(&transitions, &compiled, HISTORY)?;
            performed.extend(performed::inherited_effects(
                runtime,
                snapshot,
                &layout,
                entity,
                maximum_transitions,
                &mut facts,
                HISTORY,
            )?);
            Ok::<_, WorthQueryApplicationAttemptDenial>((effects, performed))
        })?;
        if self.facts.len().saturating_add(facts.len())
            > self
                .admission
                .allowed_graph_contract()
                .decision_fact_budget()
        {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                self.admission.operation(),
            ));
        }
        self.facts.extend(facts);
        let mut demand = PlatformEffectDemand::default();
        for effect in &effects {
            demand.observe(effect)?;
        }
        let reservation = admit_platform_effects(&self, demand)?;
        let validator_work_admission = reservation.materialize(&effects)?;
        Ok(PreparedWorkflowInstanceCancellation {
            program_revision: installed.program_revision().clone(),
            instance,
            cancellation_identity: identity,
            intent_identity,
            cancellation_identity_locator: layout.instance.cancellation_identity.clone(),
            performed: performed.into_iter().map(|effect| effect.path).collect(),
            program: WorthQueryApplicationEffectProgram {
                read_set: self,
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
                output_currentness_facts: None,
            },
        })
    }
}
