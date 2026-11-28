use crate::contracts::config::data::policies::MutationConfig;
use crate::contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog;
use crate::contracts::schema::data::RelationalSchemaRegistry;
use crate::contracts::transactions::data::outcomes::plan_artifacts::AuthoritativeApplyPlan;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

use super::effect_assembly::assemble_effect;
use super::intents::dispatch_intent;
use super::{BranchLocalDeleteAllowance, MutationEffect, MutationWorkspace};

pub struct MutationApplyOutcome {
    pub effect: MutationEffect,
    pub created_entities: crate::contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    pub created_relations: crate::contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
}

pub fn apply_plan_to_working_state(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    apply_plan: &AuthoritativeApplyPlan,
    config: &MutationConfig,
    schema_registry: &RelationalSchemaRegistry,
    aspect_plans: &AspectContractPlanCatalog,
    symbols: &mut dyn worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::mutation_input::MutationSymbolMeanings,
    branch_local_delete_allowance: BranchLocalDeleteAllowance,
    record_allocations: &mut dyn worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::allocation_batch::MutationRecordAllocations,
    commit_work_budget: Option<crate::contracts::execution::request_work::RequestWorkBudget>,
) -> Result<MutationApplyOutcome, super::MutationApplyDenial> {
    apply_plan_with_publication_edges(
        execution,
        state,
        apply_plan,
        config,
        schema_registry,
        aspect_plans,
        symbols,
        branch_local_delete_allowance,
        record_allocations,
        commit_work_budget,
        None,
    )
}

pub fn apply_plan_with_publication_edges(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    apply_plan: &AuthoritativeApplyPlan,
    config: &MutationConfig,
    schema_registry: &RelationalSchemaRegistry,
    aspect_plans: &AspectContractPlanCatalog,
    symbols: &mut dyn worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::mutation_input::MutationSymbolMeanings,
    branch_local_delete_allowance: BranchLocalDeleteAllowance,
    record_allocations: &mut dyn worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::allocation_batch::MutationRecordAllocations,
    commit_work_budget: Option<crate::contracts::execution::request_work::RequestWorkBudget>,
    deferred_publication_edges: Option<
        &std::collections::BTreeSet<crate::contracts::identity::data::record_ids::RelationId>,
    >,
) -> Result<MutationApplyOutcome, super::MutationApplyDenial> {
    let mut workspace = MutationWorkspace::new(
        *execution,
        state,
        symbols,
        config,
        schema_registry,
        aspect_plans,
        apply_plan.version_id,
        branch_local_delete_allowance,
        Some(record_allocations),
        commit_work_budget,
    );
    if let Some(edges) = deferred_publication_edges {
        workspace.defer_publication_edges(edges);
    }
    let (expected_change_count, expected_event_count) =
        estimated_mutation_effect_shape(&apply_plan.merged_intents);
    let mut effect = MutationEffect::with_capacity(expected_change_count, expected_event_count);

    for intent in &apply_plan.merged_intents {
        workspace.begin_intent(intent);
        let child = dispatch_intent(intent, &mut workspace)?;
        effect.accumulate(
            workspace.run_owned_mutation(|workspace| assemble_effect(child, workspace))?,
        );
    }

    let (created_entities, created_relations) = workspace.into_created_bindings();
    Ok(MutationApplyOutcome {
        effect,
        created_entities,
        created_relations,
    })
}

fn estimated_mutation_effect_shape(
    intents: &[crate::contracts::transactions::data::intents::mutation_intent::MutationIntent],
) -> (usize, usize) {
    use crate::contracts::transactions::data::intents::mutation_intent::CreateIntent;
    use crate::contracts::transactions::data::intents::mutation_intent::EntityMutationIntent;
    use crate::contracts::transactions::data::intents::mutation_intent::MutationIntent;
    use crate::contracts::transactions::data::intents::mutation_intent::RelationMutationIntent;

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
            MutationIntent::Entity(EntityMutationIntent::Revalidate(_))
            | MutationIntent::Relation(RelationMutationIntent::Revalidate(_)) => {}
        }
    }

    (change_count, event_count)
}
