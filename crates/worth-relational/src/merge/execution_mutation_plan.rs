mod current_snapshots;
mod materialized_value_resolution;
mod merge_client_keys;
mod source_authoritative_fields;

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};

use crate::capabilities::AspectPlanSource;
use crate::merge::data::{
    AdoptSourceRecordPlan, BoundExecutableMergeRecordPlan, CompiledMergeExecution,
    ExecutableAspectPlan, MergeExecutionMutationPlanError, PreparedMergeMutationPlan,
    ReconcileRecordPlan,
};
use crate::schema::data::LoweredAspectContractPlan;
use crate::storage::data::EntityReadRecord;
use crate::transactions::data::AspectFieldPatch;
use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, EntitySpec, MergeExecutionStructuralSummary,
    MergeExecutionSummary, MutationIntent, RelationSpec, UpdateEntityFieldsIntent,
};

use current_snapshots::current_entity_snapshot;
use materialized_value_resolution::resolved_entity_field_patch_value;
use merge_client_keys::merge_client_key;
use source_authoritative_fields::{
    entity_create_fields_from_authoritative_state, relation_create_fields_from_authoritative_state,
};

use super::MergeAccess;

struct PreparedRecordMutations {
    intents: Vec<MutationIntent>,
    summary: MergeExecutionStructuralSummary,
}

struct EntityReconcileTarget<'plan> {
    entity: EntityReadRecord,
    binding_plan: &'plan LoweredAspectContractPlan,
}

impl<'runtime> MergeAccess<'runtime> {
    pub(crate) fn derive_merge_commit_mutation_plan(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        compiled: &CompiledMergeExecution,
    ) -> Result<PreparedMergeMutationPlan, MergeExecutionMutationPlanError> {
        let mutations = self.derive_record_mutations(execution, compiled)?;
        let merge_execution_summary = self.retain_execution_summary(compiled, &mutations.summary);
        Ok(assemble_prepared_mutation_plan(
            compiled,
            mutations,
            merge_execution_summary,
        ))
    }

    fn derive_record_mutations(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        compiled: &CompiledMergeExecution,
    ) -> Result<PreparedRecordMutations, MergeExecutionMutationPlanError> {
        let mut merged_intents = Vec::new();
        let mut summary = MergeExecutionStructuralSummary {
            executed_record_count: compiled.bound_executable_plan().record_plans.len(),
            adopted_source_record_count: 0,
            preserved_shared_record_count: 0,
            reconciled_record_count: 0,
            converged_deleted_on_both_sides_count: 0,
            deleted_on_both_sides_lineage_unchanged_count: 0,
            emitted_mutation_intent_count: 0,
            emitted_entity_create_count: 0,
            emitted_relation_create_count: 0,
            emitted_entity_update_count: 0,
        };
        for record_plan in compiled.bound_executable_plan().record_plans.iter() {
            match record_plan {
                BoundExecutableMergeRecordPlan::AdoptSource(plan) => {
                    summary.adopted_source_record_count += 1;
                    let intent = self.derive_source_adoption_intent(plan)?;
                    match &intent {
                        MutationIntent::Create(CreateIntent::Entity(_)) => {
                            summary.emitted_entity_create_count += 1;
                        }
                        MutationIntent::Create(CreateIntent::Relation(_)) => {
                            summary.emitted_relation_create_count += 1;
                        }
                        _ => {}
                    }
                    summary.emitted_mutation_intent_count += 1;
                    merged_intents.push(intent);
                }
                BoundExecutableMergeRecordPlan::PreserveShared(_plan) => {
                    summary.preserved_shared_record_count += 1;
                }
                BoundExecutableMergeRecordPlan::Reconcile(plan) => {
                    summary.reconciled_record_count += 1;
                    if let Some(intent) = self.derive_reconcile_intent(execution, plan)? {
                        if matches!(
                            intent,
                            MutationIntent::Entity(EntityMutationIntent::UpdateFields(_))
                        ) {
                            summary.emitted_entity_update_count += 1;
                        }
                        summary.emitted_mutation_intent_count += 1;
                        merged_intents.push(intent);
                    }
                }
                BoundExecutableMergeRecordPlan::ConvergeDeletedOnBothSides(plan) => {
                    summary.converged_deleted_on_both_sides_count += 1;
                    if plan.lineage_continuity
                        == crate::merge::data::MergeLineageContinuityVerdict::Unchanged
                    {
                        summary.deleted_on_both_sides_lineage_unchanged_count += 1;
                    }
                }
            }
        }
        Ok(PreparedRecordMutations {
            intents: merged_intents,
            summary,
        })
    }

    fn retain_execution_summary(
        &self,
        compiled: &CompiledMergeExecution,
        summary: &MergeExecutionStructuralSummary,
    ) -> MergeExecutionSummary {
        let binding = &compiled.bound_executable_plan().authority_binding;
        let diagnostics_plan = &compiled.bound_executable_plan().diagnostics_plan;
        let correspondence_witness =
            self.retain_merge_correspondence_witness_from_planning_artifact(compiled.artifact());
        let schema_reconciliation_witness =
            self.retain_schema_reconciliation_witness_from_planning_artifact(compiled.artifact());
        let strategy_witness =
            self.retain_merge_strategy_witness_from_planning_artifact(compiled.artifact());
        let proof_packet = self.retain_merge_proof_packet_from_compiled_execution_with_witness(
            compiled,
            &correspondence_witness,
            &schema_reconciliation_witness,
            &strategy_witness,
        );
        MergeExecutionSummary {
            request: compiled.request().clone(),
            branch_basis: compiled.execution_ready_plan().basis.clone(),
            correspondence_witness,
            schema_reconciliation_witness,
            strategy_witness,
            proof_packet,
            target_head_commit_id: binding.target_head_commit_id,
            source_head_commit_id: binding.source_head_commit_id,
            merge_base_commit_id: binding.merge_base_commit_id,
            executed_record_count: summary.executed_record_count,
            adopted_source_record_count: summary.adopted_source_record_count,
            preserved_shared_record_count: summary.preserved_shared_record_count,
            reconciled_record_count: summary.reconciled_record_count,
            converged_deleted_on_both_sides_count: summary.converged_deleted_on_both_sides_count,
            deleted_on_both_sides_lineage_unchanged_count: summary
                .deleted_on_both_sides_lineage_unchanged_count,
            emitted_mutation_intent_count: summary.emitted_mutation_intent_count,
            diagnostics_digest: diagnostics_plan.digest.clone(),
            execution_digest: binding.executable_plan_digest.clone(),
        }
    }
}

fn assemble_prepared_mutation_plan(
    compiled: &CompiledMergeExecution,
    mutations: PreparedRecordMutations,
    merge_execution_summary: MergeExecutionSummary,
) -> PreparedMergeMutationPlan {
    let binding = &compiled.bound_executable_plan().authority_binding;
    PreparedMergeMutationPlan {
        target_branch: compiled.request().target_branch().clone(),
        source_branch: compiled.request().source_branch().clone(),
        merge_parent_branches: Arc::from([compiled.request().source_branch().clone()]),
        requested_merge_parent_count: 1,
        parent_commits: crate::history::data::OrderedParentList::from_authoritative(
            compiled
                .bound_executable_plan()
                .parent_order
                .iter()
                .copied()
                .collect(),
        ),
        merge_base_commits: Arc::from([binding.merge_base_commit_id]),
        merged_intents: mutations.intents,
        structural_summary: mutations.summary,
        merge_execution_summary,
    }
}

impl<'runtime> MergeAccess<'runtime> {
    fn derive_source_adoption_intent(
        &self,
        plan: &AdoptSourceRecordPlan,
    ) -> Result<MutationIntent, MergeExecutionMutationPlanError> {
        match &plan.source_visible_snapshot {
            crate::merge::data::VisibleMergeRecordSnapshot::Entity(entity) => {
                Ok(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: entity.entity_id.partition_id,
                    kind_id: entity.kind.kind_id,
                    client_key: merge_client_key("adopt-source-entity", &plan.source_record),
                    fields: entity_create_fields_from_authoritative_state(
                        self.runtime,
                        plan,
                        entity,
                    )?,
                })))
            }
            crate::merge::data::VisibleMergeRecordSnapshot::Relation(relation) => Ok(
                MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                    partition_id: relation.relation_id.partition_id,
                    kind_id: relation.kind.kind_id,
                    client_key: merge_client_key("adopt-source-relation", &plan.source_record),
                    source: crate::transactions::data::EntityReference::Existing(relation.source),
                    target: crate::transactions::data::EntityReference::Existing(relation.target),
                    fields: relation_create_fields_from_authoritative_state(
                        self.runtime,
                        plan,
                        relation,
                    )?,
                })),
            ),
        }
    }
}

impl<'runtime> MergeAccess<'runtime> {
    fn derive_reconcile_intent(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: &ReconcileRecordPlan,
    ) -> Result<Option<MutationIntent>, MergeExecutionMutationPlanError> {
        match (&plan.source_visible_snapshot, &plan.target_record) {
            (
                crate::merge::data::VisibleMergeRecordSnapshot::Entity(source_entity),
                crate::transactions::data::RecordRef::Entity(target_entity_id),
            ) => self.derive_entity_reconcile_intent(
                execution,
                plan,
                source_entity,
                *target_entity_id,
            ),
            (
                crate::merge::data::VisibleMergeRecordSnapshot::Relation(_),
                crate::transactions::data::RecordRef::Relation(_),
            ) => Err(
                MergeExecutionMutationPlanError::UnsupportedReconcileRecordKind {
                    record: plan.target_record.clone(),
                    detail: "relation reconciliation is not executable in phase D",
                },
            ),
            _ => Err(
                MergeExecutionMutationPlanError::UnsupportedReconcileRecordKind {
                    record: plan.target_record.clone(),
                    detail: "source/target record kinds do not match reconcile executable class",
                },
            ),
        }
    }

    fn derive_entity_reconcile_intent(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: &ReconcileRecordPlan,
        source_entity: &EntityReadRecord,
        target_entity_id: crate::identity::data::EntityId,
    ) -> Result<Option<MutationIntent>, MergeExecutionMutationPlanError> {
        let target = self.entity_reconcile_target(execution, plan, target_entity_id)?;
        let resolved_fields = resolved_entity_reconcile_fields(plan, source_entity, &target)?;
        Ok(entity_reconcile_intent(target_entity_id, resolved_fields))
    }

    fn entity_reconcile_target<'plan>(
        &'plan self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: &ReconcileRecordPlan,
        target_entity_id: crate::identity::data::EntityId,
    ) -> Result<EntityReconcileTarget<'plan>, MergeExecutionMutationPlanError> {
        let entity = current_entity_snapshot(self.runtime, execution, target_entity_id)
            .map_err(MergeExecutionMutationPlanError::Read)?
            .ok_or_else(
                || MergeExecutionMutationPlanError::MissingTargetEntitySnapshot {
                    record: plan.target_record.clone(),
                },
            )?;
        let binding_plan = self
            .runtime
            .entity_aspect_plan(entity.kind.kind_id)
            .ok_or_else(
                || MergeExecutionMutationPlanError::UnsupportedReconcileRecordKind {
                    record: plan.target_record.clone(),
                    detail: "target entity kind has no executable aspect plan",
                },
            )?;
        Ok(EntityReconcileTarget {
            entity,
            binding_plan,
        })
    }
}

fn resolved_entity_reconcile_fields(
    plan: &ReconcileRecordPlan,
    source_entity: &EntityReadRecord,
    target: &EntityReconcileTarget<'_>,
) -> Result<BTreeMap<AspectFieldLocator, AspectValue>, MergeExecutionMutationPlanError> {
    let mut resolved_fields = BTreeMap::<AspectFieldLocator, AspectValue>::new();

    for aspect_plan in plan.aspect_plan.iter() {
        let ExecutableAspectPlan::ReconcileValue {
            aspect_key,
            resolved_value,
            ..
        } = aspect_plan
        else {
            continue;
        };
        let binding = target
                .binding_plan
                .executable_bindings
                .iter()
                .find(|binding| binding.aspect_key() == aspect_key)
                .ok_or_else(|| MergeExecutionMutationPlanError::UnsupportedAspectMutationMaterialization {
                    record: plan.target_record.clone(),
                    aspect_key: aspect_key.clone(),
                    detail: "target entity executable binding missing during reconcile mutation derivation",
                })?;
        let resolved_value = resolved_value.as_ref().ok_or_else(|| {
            MergeExecutionMutationPlanError::MissingResolvedAspectValue {
                record: plan.target_record.clone(),
                aspect_key: aspect_key.clone(),
            }
        })?;
        if let Some((field, value)) = resolved_entity_field_patch_value(
            plan,
            source_entity,
            &target.entity,
            binding,
            aspect_key,
            resolved_value,
        )? {
            resolved_fields.insert(field, value);
        }
    }
    Ok(resolved_fields)
}

fn entity_reconcile_intent(
    target_entity_id: crate::identity::data::EntityId,
    fields: BTreeMap<AspectFieldLocator, AspectValue>,
) -> Option<MutationIntent> {
    (!fields.is_empty()).then(|| {
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: target_entity_id,
                fields: AspectFieldPatch::from(fields),
            },
        ))
    })
}
