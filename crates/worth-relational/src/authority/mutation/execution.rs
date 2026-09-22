use crate::config::data::MutationConfig;
use crate::schema::data::{AspectContractPlanCatalog, RelationalSchemaRegistry};
use crate::storage::overlay::WorkingState;
use crate::symbols::data::StringInterner;
use crate::transactions::data::{AuthoritativeApplyPlan, CommitConflict};

use super::effect_assembly::assemble_effect;
use super::intents::dispatch_intent;
use super::{
    BranchLocalDeleteAllowance, MutationEffect, MutationPreparationTelemetry, MutationWorkspace,
};

pub(crate) struct MutationApplyOutcome {
    pub(crate) effect: MutationEffect,
    pub(crate) preparation_telemetry: MutationPreparationTelemetry,
    pub(crate) created_entities: crate::transactions::data::CommitCreatedEntityBindings,
    pub(crate) created_relations: crate::transactions::data::CommitCreatedRelationBindings,
}

pub(crate) fn apply_plan_to_working_state(
    state: &mut WorkingState,
    apply_plan: &AuthoritativeApplyPlan,
    config: &MutationConfig,
    schema_registry: &RelationalSchemaRegistry,
    aspect_plans: &AspectContractPlanCatalog,
    symbols: &mut StringInterner,
    branch_local_delete_allowance: BranchLocalDeleteAllowance,
    record_allocations: &mut crate::runtime::PendingRecordAllocations,
) -> Result<MutationApplyOutcome, CommitConflict> {
    let mut workspace = MutationWorkspace::new(
        state,
        symbols,
        config,
        schema_registry,
        aspect_plans,
        apply_plan.version_id,
        branch_local_delete_allowance,
        Some(record_allocations),
    );
    let (expected_change_count, expected_event_count) =
        estimated_mutation_effect_shape(&apply_plan.merged_intents);
    let mut effect = MutationEffect::with_capacity(expected_change_count, expected_event_count);

    for intent in &apply_plan.merged_intents {
        let child = dispatch_intent(intent, &mut workspace)?;
        effect.accumulate(assemble_effect(child, &mut workspace)?);
    }

    let preparation_telemetry = workspace.preparation_telemetry();
    let (created_entities, created_relations) = workspace.into_created_bindings();
    record_allocations
        .finish_mutation()
        .map_err(record_allocation_conflict)?;
    Ok(MutationApplyOutcome {
        effect,
        preparation_telemetry,
        created_entities,
        created_relations,
    })
}

fn record_allocation_conflict(
    denial: crate::transactions::data::RecordAllocationDenial,
) -> CommitConflict {
    CommitConflict::new(crate::transactions::data::ConflictClass::RecordAllocationDenied { denial })
}

fn estimated_mutation_effect_shape(
    intents: &[crate::transactions::data::MutationIntent],
) -> (usize, usize) {
    use crate::transactions::data::{
        CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent,
    };

    let mut change_count = 0usize;
    let mut event_count = 0usize;

    for intent in intents {
        match intent {
            MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
                change_count += spec.field_patches.len();
                event_count += 1;
            }
            MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
                change_count += spec.endpoints.len();
                event_count += 1;
            }
            MutationIntent::Create(CreateIntent::Entity(_))
            | MutationIntent::Create(CreateIntent::EntityAspects(_))
            | MutationIntent::Create(CreateIntent::Relation(_))
            | MutationIntent::Create(CreateIntent::RelationAspects(_))
            | MutationIntent::Entity(EntityMutationIntent::UpdateFields(_))
            | MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(_))
            | MutationIntent::Entity(EntityMutationIntent::Delete(_))
            | MutationIntent::Entity(EntityMutationIntent::Replace(_))
            | MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(_))
            | MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(_))
            | MutationIntent::Relation(RelationMutationIntent::Delete(_)) => {
                change_count += 1;
                event_count += 1;
            }
            MutationIntent::Materialization(_) => {
                change_count += 1;
                event_count += 1;
            }
            // A revalidation demand rewrites nothing, so it contributes neither a
            // change to deliver nor an event to record.
            MutationIntent::Entity(EntityMutationIntent::Revalidate(_)) => {}
        }
    }

    (change_count, event_count)
}
