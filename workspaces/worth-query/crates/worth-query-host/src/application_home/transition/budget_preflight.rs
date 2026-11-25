//! Exact pre-intent target-step budget admission over the full predecessor
//! root, target-step created records, and installed target index inventory.
//!
//! Runs strictly before Store publishes any durable `TransitionPending` intent,
//! preventing an insufficient index maintenance budget from stranding the
//! predecessor Home on disk.

use std::collections::BTreeSet;

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::application_home_integration::PreparedHomeProgramTransitionTarget;
use worth_relational::facade::{
    identity::PartitionId,
    indexes::{
        DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
        DerivedIndexMaintenanceDenialKind, DerivedIndexMaintenanceWork,
    },
    runtime::RelationalRuntime,
    transactions::{CreateIntent, MutationIntent},
};

use super::{
    plan::WorthQueryHomeProgramTransitionPlan,
    refusal::WorthQueryHomeProgramTransitionDenial as Denial,
};

#[derive(Default)]
struct TargetPlanDelta {
    created_entities: usize,
    created_relations: usize,
    touched_partitions: BTreeSet<PartitionId>,
}

fn accumulate_target_plan_delta(plan: &WorthQueryHomeProgramTransitionPlan) -> TargetPlanDelta {
    let mut delta = TargetPlanDelta::default();
    let target_end = (plan.target as usize).min(plan.steps.len().saturating_sub(1));
    for step in &plan.steps[..=target_end] {
        for batch in &step.batches {
            for intent in &batch.intents {
                let MutationIntent::Create(create) = intent else {
                    continue;
                };
                match create {
                    CreateIntent::Entity(spec) => {
                        delta.created_entities = delta.created_entities.saturating_add(1);
                        delta.touched_partitions.insert(spec.partition_id);
                    }
                    CreateIntent::EntityAspects(spec) => {
                        delta.created_entities = delta.created_entities.saturating_add(1);
                        delta.touched_partitions.insert(spec.partition_id);
                    }
                    CreateIntent::BulkEntities(spec) => {
                        delta.created_entities = delta
                            .created_entities
                            .saturating_add(spec.client_keys.len());
                        delta.touched_partitions.insert(spec.partition_id);
                    }
                    CreateIntent::Relation(spec) => {
                        delta.created_relations = delta.created_relations.saturating_add(1);
                        delta.touched_partitions.insert(spec.partition_id);
                    }
                    CreateIntent::RelationAspects(spec) => {
                        delta.created_relations = delta.created_relations.saturating_add(1);
                        delta.touched_partitions.insert(spec.partition_id);
                    }
                    CreateIntent::BulkRelations(spec) => {
                        delta.created_relations = delta
                            .created_relations
                            .saturating_add(spec.client_keys.len());
                        delta.touched_partitions.insert(spec.partition_id);
                    }
                }
            }
        }
    }
    delta
}

/// Admit `resources` against the full successor root's cold-reconstruction
/// footprint (entity slots + relation slots + regions + target-step creations +
/// installed target indexes) before Store reserves or publishes any durable
/// `TransitionPending` intent.
pub(super) fn preflight_fresh_target_budget<Schema: ApplicationSchemaComposition, Program>(
    native: &RelationalRuntime,
    plan: &WorthQueryHomeProgramTransitionPlan,
    target: &PreparedHomeProgramTransitionTarget<Schema, Program>,
    resources: DerivedIndexMaintenanceBudget,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let stats = native
        .storage_access()
        .partition_storage_stats(request)
        .map_err(|cause| {
            Denial::Indexes(DerivedIndexMaintenanceDenial {
                kind: DerivedIndexMaintenanceDenialKind::Execution(cause),
                work: DerivedIndexMaintenanceWork::default(),
            })
        })?;

    let mut current_entity_slots = 0usize;
    let mut current_relation_slots = 0usize;
    let mut current_live_entities = 0usize;
    let mut current_live_relations = 0usize;
    let mut partitions: BTreeSet<PartitionId> = BTreeSet::new();
    for partition in &stats {
        partitions.insert(partition.partition_id);
        current_entity_slots = current_entity_slots.saturating_add(partition.entity_slots);
        current_relation_slots = current_relation_slots.saturating_add(partition.relation_slots);
        current_live_entities = current_live_entities.saturating_add(partition.live_entities);
        current_live_relations = current_live_relations.saturating_add(partition.live_relations);
    }

    let delta = accumulate_target_plan_delta(plan);
    partitions.extend(delta.touched_partitions);

    let after_entity_slots = current_entity_slots.saturating_add(delta.created_entities);
    let after_relation_slots = current_relation_slots.saturating_add(delta.created_relations);
    let after_regions = partitions.len();
    let required_cold_slots = after_entity_slots
        .saturating_add(after_relation_slots)
        .saturating_add(after_regions);

    if resources
        .maximum_cold_record_slots
        .is_some_and(|maximum| maximum == 0 || required_cold_slots > maximum)
    {
        return Err(Denial::ColdSlotBudgetExceeded(
            DerivedIndexMaintenanceDenial {
                kind: DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired,
                work: DerivedIndexMaintenanceWork {
                    cold_record_slots: required_cold_slots,
                    ..DerivedIndexMaintenanceWork::default()
                },
            },
        ));
    }

    let after_live_entities = current_live_entities.saturating_add(delta.created_entities);
    let after_live_relations = current_live_relations.saturating_add(delta.created_relations);
    let indexes = target.installed_index_counts();

    let max_entity_rows = indexes.entity_field.saturating_mul(after_live_entities);
    let max_relation_rows = indexes
        .relation_field
        .saturating_add(indexes.related_ordering)
        .saturating_mul(after_live_relations);
    let max_join_rows = indexes
        .relation_join
        .saturating_mul(after_live_relations.saturating_mul(after_live_relations));
    let required_derived_rows = max_entity_rows
        .saturating_add(max_relation_rows)
        .saturating_add(max_join_rows);

    let join_and_ordering_reads = indexes
        .related_ordering
        .saturating_add(indexes.relation_join)
        .saturating_mul(after_live_relations.saturating_mul(3));
    let required_work_units = required_cold_slots
        .saturating_add(after_live_entities)
        .saturating_add(after_live_relations)
        .saturating_add(join_and_ordering_reads)
        .saturating_add(required_derived_rows);

    let exceeds_rows = resources
        .maximum_derived_rows
        .is_some_and(|maximum| required_derived_rows > maximum);
    let exceeds_work = resources
        .maximum_work_units
        .is_some_and(|maximum| required_work_units > maximum);

    if exceeds_rows || exceeds_work {
        return Err(Denial::Indexes(DerivedIndexMaintenanceDenial {
            kind: DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded,
            work: DerivedIndexMaintenanceWork {
                cold_record_slots: required_cold_slots,
                work_units: required_work_units,
                derived_rows: required_derived_rows,
                ..DerivedIndexMaintenanceWork::default()
            },
        }));
    }

    Ok(())
}
