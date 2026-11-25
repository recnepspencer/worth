//! Pre-intent target-step budget admission delegating directly to Native's
//! exact cold-reconstruction preflight owner.
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
    indexes::DerivedIndexMaintenanceBudget,
    runtime::RelationalRuntime,
    transactions::{CreateIntent, MutationIntent},
};
use worth_runtime_world::facade::ProductBranchObservation;

use super::{
    plan::WorthQueryHomeProgramTransitionPlan,
    refusal::WorthQueryHomeProgramTransitionDenial as Denial,
};

#[derive(Default)]
struct TargetPlanDelta {
    created_entities: usize,
    created_relations: usize,
    touched_partitions: Vec<PartitionId>,
}

fn accumulate_target_plan_delta(plan: &WorthQueryHomeProgramTransitionPlan) -> TargetPlanDelta {
    let mut created_entities = 0usize;
    let mut created_relations = 0usize;
    let mut touched = BTreeSet::new();
    let target_end = (plan.target as usize).min(plan.steps.len().saturating_sub(1));
    for step in &plan.steps[..=target_end] {
        for batch in &step.batches {
            for intent in &batch.intents {
                let MutationIntent::Create(create) = intent else {
                    continue;
                };
                match create {
                    CreateIntent::Entity(spec) => {
                        created_entities = created_entities.saturating_add(1);
                        touched.insert(spec.partition_id);
                    }
                    CreateIntent::EntityAspects(spec) => {
                        created_entities = created_entities.saturating_add(1);
                        touched.insert(spec.partition_id);
                    }
                    CreateIntent::BulkEntities(spec) => {
                        created_entities = created_entities.saturating_add(spec.client_keys.len());
                        touched.insert(spec.partition_id);
                    }
                    CreateIntent::Relation(spec) => {
                        created_relations = created_relations.saturating_add(1);
                        touched.insert(spec.partition_id);
                    }
                    CreateIntent::RelationAspects(spec) => {
                        created_relations = created_relations.saturating_add(1);
                        touched.insert(spec.partition_id);
                    }
                    CreateIntent::BulkRelations(spec) => {
                        created_relations =
                            created_relations.saturating_add(spec.client_keys.len());
                        touched.insert(spec.partition_id);
                    }
                }
            }
        }
    }
    TargetPlanDelta {
        created_entities,
        created_relations,
        touched_partitions: touched.into_iter().collect(),
    }
}

/// Admit `resources` through Native's exact cold-reconstruction preflight owner
/// before Store reserves or publishes any durable `TransitionPending` intent.
pub(super) fn preflight_fresh_target_budget<Schema: ApplicationSchemaComposition, Program>(
    native: &RelationalRuntime,
    source: &ProductBranchObservation,
    plan: &WorthQueryHomeProgramTransitionPlan,
    target: &PreparedHomeProgramTransitionTarget<Schema, Program>,
    resources: DerivedIndexMaintenanceBudget,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let delta = accumulate_target_plan_delta(plan);
    native
        .preflight_home_program_target_index_budget(
            request,
            source.basis().relational_basis(),
            target.native_contracts(),
            delta.created_entities,
            delta.created_relations,
            &delta.touched_partitions,
            resources,
        )
        .map(|_| ())
        .map_err(Denial::Indexes)
}
