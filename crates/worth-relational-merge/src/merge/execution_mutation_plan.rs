pub mod current_snapshots;
pub mod materialized_value_resolution;
pub mod merge_client_keys;
pub mod source_authoritative_fields;

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};

use crate::contracts::schema::data::aspect_contracts::lowered_plan::{
    AspectContractPlanCatalog, LoweredAspectContractPlan,
};
use crate::contracts::storage::data::read_records::EntityReadRecord;
use crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch;
use crate::contracts::transactions::data::intents::mutation_intent::{
    CreateIntent, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
};
use crate::contracts::transactions::data::outcomes::plan_artifacts::{
    MergeExecutionStructuralSummary, MergeExecutionSummary,
};
use crate::contracts::transactions::data::primitives::{EntitySpec, RelationSpec};
use crate::merge::data::{
    AdoptSourceRecordPlan, BoundExecutableMergeRecordPlan, CompiledMergeExecution,
    ExecutableAspectPlan, MergeExecutionMutationPlanError, PreparedMergeMutationPlan,
    ReconcileRecordPlan,
};

use materialized_value_resolution::resolved_entity_field_patch_value;
use merge_client_keys::merge_client_key;
use source_authoritative_fields::{
    entity_create_fields_from_authoritative_state, relation_create_fields_from_authoritative_state,
};

use super::correspondence_witness::retain_merge_correspondence_witness_from_planning_artifact;
use super::proof_packet::retain_merge_proof_packet_from_compiled_execution_with_witness;
pub use current_snapshots::current_entity_snapshot;

type CurrentEntityResult = Result<
    Option<EntityReadRecord>,
    crate::contracts::execution::denial_cause::RelationalExecutionDenialCause,
>;

struct PreparedRecordMutations {
    intents: Vec<MutationIntent>,
    summary: MergeExecutionStructuralSummary,
}

struct EntityReconcileTarget<'plan> {
    entity: EntityReadRecord,
    binding_plan: &'plan LoweredAspectContractPlan,
}

pub fn derive_merge_commit_mutation_plan(
    aspect_plans: &AspectContractPlanCatalog,
    read_current_entity: &mut dyn FnMut(
        crate::contracts::identity::data::record_ids::EntityId,
    ) -> CurrentEntityResult,
    compiled: &CompiledMergeExecution,
) -> Result<PreparedMergeMutationPlan, MergeExecutionMutationPlanError> {
    let mutations = derive_record_mutations(aspect_plans, read_current_entity, compiled)?;
    let merge_execution_summary = retain_execution_summary(compiled, &mutations.summary);
    Ok(assemble_prepared_mutation_plan(
        compiled,
        mutations,
        merge_execution_summary,
    ))
}

fn derive_record_mutations(
    aspect_plans: &AspectContractPlanCatalog,
    read_current_entity: &mut dyn FnMut(
        crate::contracts::identity::data::record_ids::EntityId,
    ) -> CurrentEntityResult,
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
                let intent = derive_source_adoption_intent(aspect_plans, plan)?;
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
                if let Some(intent) =
                    derive_reconcile_intent(aspect_plans, read_current_entity, plan)?
                {
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
    compiled: &CompiledMergeExecution,
    summary: &MergeExecutionStructuralSummary,
) -> MergeExecutionSummary {
    let binding = &compiled.bound_executable_plan().authority_binding;
    let diagnostics_plan = &compiled.bound_executable_plan().diagnostics_plan;
    let correspondence_witness =
        retain_merge_correspondence_witness_from_planning_artifact(compiled.artifact());
    let schema_reconciliation_witness = compiled.artifact().schema_reconciliation_witness.clone();
    let strategy_witness = compiled.artifact().strategy_witness.clone();
    let proof_packet = retain_merge_proof_packet_from_compiled_execution_with_witness(
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
        parent_commits:
            crate::contracts::history::data::commit_receipt::OrderedParentList::from_authoritative(
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

fn derive_source_adoption_intent(
    aspect_plans: &AspectContractPlanCatalog,
    plan: &AdoptSourceRecordPlan,
) -> Result<MutationIntent, MergeExecutionMutationPlanError> {
    match &plan.source_visible_snapshot {
        crate::merge::data::VisibleMergeRecordSnapshot::Entity(entity) => {
            Ok(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: entity.entity_id.partition_id,
                kind_id: entity.kind.kind_id,
                client_key: merge_client_key("adopt-source-entity", &plan.source_record),
                fields: entity_create_fields_from_authoritative_state(aspect_plans, plan, entity)?,
            })))
        }
        crate::merge::data::VisibleMergeRecordSnapshot::Relation(relation) => Ok(
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: relation.relation_id.partition_id,
                kind_id: relation.kind.kind_id,
                client_key: merge_client_key("adopt-source-relation", &plan.source_record),
                source: crate::contracts::transactions::data::primitives::EntityReference::Existing(
                    relation.source,
                ),
                target: crate::contracts::transactions::data::primitives::EntityReference::Existing(
                    relation.target,
                ),
                fields: relation_create_fields_from_authoritative_state(
                    aspect_plans,
                    plan,
                    relation,
                )?,
            })),
        ),
    }
}

fn derive_reconcile_intent(
    aspect_plans: &AspectContractPlanCatalog,
    read_current_entity: &mut dyn FnMut(
        crate::contracts::identity::data::record_ids::EntityId,
    ) -> CurrentEntityResult,
    plan: &ReconcileRecordPlan,
) -> Result<Option<MutationIntent>, MergeExecutionMutationPlanError> {
    match (&plan.source_visible_snapshot, &plan.target_record) {
        (
            crate::merge::data::VisibleMergeRecordSnapshot::Entity(source_entity),
            crate::contracts::transactions::data::primitives::RecordRef::Entity(target_entity_id),
        ) => derive_entity_reconcile_intent(
            aspect_plans,
            read_current_entity,
            plan,
            source_entity,
            *target_entity_id,
        ),
        (
            crate::merge::data::VisibleMergeRecordSnapshot::Relation(_),
            crate::contracts::transactions::data::primitives::RecordRef::Relation(_),
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
    aspect_plans: &AspectContractPlanCatalog,
    read_current_entity: &mut dyn FnMut(
        crate::contracts::identity::data::record_ids::EntityId,
    ) -> CurrentEntityResult,
    plan: &ReconcileRecordPlan,
    source_entity: &EntityReadRecord,
    target_entity_id: crate::contracts::identity::data::record_ids::EntityId,
) -> Result<Option<MutationIntent>, MergeExecutionMutationPlanError> {
    let target =
        entity_reconcile_target(aspect_plans, read_current_entity, plan, target_entity_id)?;
    let resolved_fields = resolved_entity_reconcile_fields(plan, source_entity, &target)?;
    Ok(entity_reconcile_intent(target_entity_id, resolved_fields))
}

fn entity_reconcile_target<'plan>(
    aspect_plans: &'plan AspectContractPlanCatalog,
    read_current_entity: &mut dyn FnMut(
        crate::contracts::identity::data::record_ids::EntityId,
    ) -> CurrentEntityResult,
    plan: &ReconcileRecordPlan,
    target_entity_id: crate::contracts::identity::data::record_ids::EntityId,
) -> Result<EntityReconcileTarget<'plan>, MergeExecutionMutationPlanError> {
    let entity = read_current_entity(target_entity_id)
        .map_err(MergeExecutionMutationPlanError::Read)?
        .ok_or_else(
            || MergeExecutionMutationPlanError::MissingTargetEntitySnapshot {
                record: plan.target_record.clone(),
            },
        )?;
    let binding_plan = aspect_plans
        .entity_plans
        .get(&entity.kind.kind_id)
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
    target_entity_id: crate::contracts::identity::data::record_ids::EntityId,
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
