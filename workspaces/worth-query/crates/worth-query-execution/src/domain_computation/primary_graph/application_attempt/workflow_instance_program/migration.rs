//! An explicit migration publishes a successor instance on the target
//! definition, resumed at the requested node, and ends the source in the same
//! commit. [`admit_workflow_migration`] decides whether the move is lawful.
//!
//! A fork continuation is the same succession for a fork's copy of an
//! instance started on another branch. It ends only that copy, on the fork;
//! the instance on its own branch is untouched.

mod lineage;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::transactions::EntityReference;

use super::super::effect_program::{admit_platform_effects, PlatformEffectDemand};
use super::super::{
    PublishedWorkflowDefinitionRef, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationObservedFact, WorthQueryApplicationRealizedEffect,
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};
use super::{intent_identity, PreparedWorkflowInstanceStart, PublishedWorkflowInstanceRef};
use crate::domain_computation::primary_graph::workflow::{
    definition::{
        reconstruct_compiled_definition, CompiledWorkflowDefinition,
        WorkflowDefinitionCompilationPosture,
    },
    instance::{
        admit_workflow_migration, visit_instance_start_facts, WorkflowInstanceState,
        WorkflowPerformedEffect, WorkflowSuccession,
    },
    schema::WorthQueryWorkflowLayout,
};
use lineage::{inherited_effects, successor_identity};

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
    pub(in crate::domain_computation::primary_graph) fn materialize_workflow_instance_migration<
        Capability,
        Spec,
        Program,
    >(
        mut self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        succession: WorkflowSuccession,
        source: PublishedWorkflowInstanceRef,
        target: PublishedWorkflowDefinitionRef,
        resume_at: &str,
        start_key_identity: [u8; 32],
    ) -> Result<
        PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let branch = self.lease.product().product_branch();
        let fork = match succession {
            WorkflowSuccession::Migration => {
                self.authorize_instance_start::<Capability, Spec, Program>(
                    installed,
                    source.branch(),
                )?;
                self.authorize_instance_start::<Capability, Spec, Program>(
                    installed,
                    target.branch(),
                )?;
                None
            }
            // The fork's copy records the branch it was started on, and the
            // target names a definition the fork holds: its own, or one it
            // copied from that branch. Fork truth decides both below.
            WorkflowSuccession::ForkContinuation => {
                self.authorize_instance_start::<Capability, Spec, Program>(installed, branch)?;
                // An instance on its own branch moves only by migration.
                let own_branch = source.branch() == branch;
                // A definition from any third branch is not one the fork holds.
                let foreign_target =
                    target.branch() != branch && target.branch() != source.branch();
                if own_branch || foreign_target {
                    return Err(WorthQueryApplicationAttemptDenial::new(
                        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch,
                        self.admission.operation(),
                    ));
                }
                Some(branch.occurrence_ordinal())
            }
        };
        let source = source.copied_onto(branch);
        let target = PublishedWorkflowDefinitionRef::retained(
            branch,
            target.entity_id(),
            target.content_identity().clone(),
        );
        let layout = self.lease.layout.workflow().clone();
        let compile = |published: &PublishedWorkflowDefinitionRef, posture| {
            reconstruct_compiled_definition(
                self.lease.handle(),
                self.lease.snapshot(),
                &layout,
                published,
                installed.program_revision(),
                Spec::IDENTITY.as_str(),
                installed.support_identity_bytes(),
                usize::from(installed.resources().maximum_definition_nodes()),
                usize::from(installed.resources().maximum_definition_connections()),
                posture,
            )
        };
        let (mut from, mut facts) = compile(
            &PublishedWorkflowDefinitionRef::retained(
                source.branch(),
                source.definition_entity_id(),
                source.definition_content_identity().clone(),
            ),
            WorkflowDefinitionCompilationPosture::Retained,
        )?;
        let (to, target_facts) = compile(&target, WorkflowDefinitionCompilationPosture::Current)?;
        facts.extend(target_facts);
        // The identity needs only the resumed target, so a replay derives it
        // without re-admitting a source that has already ended.
        let resumed = to
            .resumed_at(resume_at)
            .ok_or_else(|| unmapped("the resume node is not in the target definition"))?;
        let subject = self.admission.scope_entity_id();
        let (instance_identity, instance_intent_identity) = intent_identity::migration_identity(
            &resumed,
            source.entity_id(),
            fork,
            subject,
            start_key_identity,
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
        let handle = self.lease.handle();
        let snapshot = self.lease.snapshot();
        let observed = handle.with_runtime(|runtime| {
            let mut observed =
                super::super::workflow_instance_observation::observe_workflow_instance(
                    handle,
                    runtime,
                    snapshot,
                    &layout,
                    &source,
                    subject,
                    from.lineage(),
                    &mut from,
                    maximum_transitions,
                    installed.resources().history_reconstruction_budget(),
                )?;
            observed.ensure_history(
                handle,
                runtime,
                snapshot,
                &layout,
                source.entity_id(),
                maximum_transitions,
                &from,
            )?;
            let inherited = inherited_effects(
                runtime,
                snapshot,
                &layout,
                source.entity_id(),
                maximum_transitions,
                &mut facts,
            )?;
            Ok::<_, WorthQueryApplicationAttemptDenial>((observed, inherited))
        });
        let effects = match observed {
            // A source this request already migrated emits nothing. Its false
            // readiness fact never becomes true again, so the program resolves
            // the exact replay; it never commits empty. Any other request for
            // a migrated source is refused as migrated.
            Err(ended)
                if ended.kind()
                    == WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrated =>
            {
                let successor = handle.with_runtime(|runtime| {
                    successor_identity(runtime, snapshot, &layout, source.entity_id())
                });
                if successor.as_deref() != Some(instance_identity.as_str()) {
                    return Err(ended);
                }
                facts.push(WorthQueryApplicationObservedFact::Field {
                    entity_id: source.entity_id(),
                    kind: layout.instance.entity_kind,
                    locator: layout.instance.state.clone(),
                    value: AspectValue::UInt64(WorkflowInstanceState::Ready.persisted_tag()),
                });
                Vec::new()
            }
            Err(denial) => return Err(denial),
            Ok((observed, inherited)) => {
                let Some(live_membership) = observed.live_membership else {
                    return Err(unmapped("a completed instance has no work left to migrate"));
                };
                let settled = observed
                    .transitions
                    .iter()
                    .map(|transition| transition.settlement)
                    .collect::<Vec<_>>();
                let mut performed = observed
                    .transitions
                    .iter()
                    .filter(|transition| {
                        transition.settlement.operation_receipt_identity().is_some()
                    })
                    .map(|transition| {
                        from.node(transition.settlement.node())
                            .map(|node| WorkflowPerformedEffect {
                                transition: transition.entity,
                                path: node.path().to_owned(),
                            })
                            .ok_or_else(|| unmapped("a performed effect is not in its definition"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                performed.extend(inherited);
                admit_workflow_migration(succession, &from, &resumed, &settled, &performed)?;
                facts.extend(observed.facts);
                successor_effects(
                    &layout,
                    &resumed,
                    &instance_identity,
                    self.lease.product().product_branch().occurrence_ordinal(),
                    subject,
                    (source.entity_id(), live_membership),
                    &performed,
                )?
            }
        };
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
        Ok(PreparedWorkflowInstanceStart {
            program_revision: resumed.program_revision().clone(),
            definition: resumed.definition(),
            definition_content_identity: resumed.content_identity().clone(),
            instance_identity,
            instance_intent_identity,
            instance_identity_locator: layout.instance.identity.clone(),
            start_path: resumed.start_path().to_owned(),
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

/// The successor's own start facts, its link to the source and to every
/// effect it carries, and the source's end. Live membership moves from the
/// source to the successor, so lineage capacity is unchanged.
fn successor_effects(
    layout: &WorthQueryWorkflowLayout,
    resumed: &CompiledWorkflowDefinition,
    instance_identity: &str,
    branch_occurrence: u64,
    subject: EntityId,
    (source, live_membership): (EntityId, RelationId),
    performed: &[WorkflowPerformedEffect],
) -> Result<Vec<WorthQueryApplicationRealizedEffect>, WorthQueryApplicationAttemptDenial> {
    let mut effects = Vec::new();
    let successor = visit_instance_start_facts(
        layout,
        resumed,
        instance_identity,
        branch_occurrence,
        subject,
        |effect| {
            effects.push(effect);
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        },
    )?;
    effects.push(WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: layout.instance_migrated_from_relation,
        key: "instance-migrated-from".to_owned(),
        from: EntityReference::Created(successor.clone()),
        to: EntityReference::Existing(source),
    });
    for (ordinal, effect) in performed.iter().enumerate() {
        effects.push(WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: layout.instance_prior_effect_relation,
            key: format!("instance-prior-effect-{ordinal}"),
            from: EntityReference::Created(successor.clone()),
            to: EntityReference::Existing(effect.transition),
        });
    }
    effects.push(WorthQueryApplicationRealizedEffect::UpdateEntity {
        entity: "workflow-instance".to_owned(),
        entity_id: source,
        fields: std::collections::BTreeMap::from([(
            layout.instance.state.clone(),
            AspectValue::UInt64(WorkflowInstanceState::Migrated.persisted_tag()),
        )]),
    });
    effects.push(WorthQueryApplicationRealizedEffect::DeleteRelation {
        relation_id: live_membership,
    });
    Ok(effects)
}

fn unmapped(subject: &'static str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped,
        subject,
    )
}
