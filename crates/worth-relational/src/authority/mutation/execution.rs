mod original_adjacency;
use crate::config::data::MutationConfig;
use crate::schema::data::{AspectContractPlanCatalog, RelationalSchemaRegistry};
use crate::storage::overlay::WorkingState;
use crate::transactions::data::AuthoritativeApplyPlan;

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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    apply_plan: &AuthoritativeApplyPlan,
    config: &MutationConfig,
    schema_registry: &RelationalSchemaRegistry,
    aspect_plans: &AspectContractPlanCatalog,
    symbols: &mut dyn crate::runtime::MutationSymbolMeanings,
    branch_local_delete_allowance: BranchLocalDeleteAllowance,
    record_allocations: &mut dyn crate::runtime::MutationRecordAllocations,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    commit_work_budget: Option<crate::execution::RequestWorkBudget>,
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
    let (expected_change_count, expected_event_count) =
        estimated_mutation_effect_shape(&apply_plan.merged_intents);
    let mut effect = MutationEffect::with_capacity(expected_change_count, expected_event_count);

    for intent in &apply_plan.merged_intents {
        let child = dispatch_intent(intent, &mut workspace, lease)?;
        effect.accumulate(
            workspace.run_owned_mutation(|workspace| assemble_effect(child, workspace))?,
        );
    }

    let preparation_telemetry = workspace.preparation_telemetry();
    let (created_entities, created_relations) = workspace.into_created_bindings();
    Ok(MutationApplyOutcome {
        effect,
        preparation_telemetry,
        created_entities,
        created_relations,
    })
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
            MutationIntent::Entity(EntityMutationIntent::Revalidate(_))
            | MutationIntent::Relation(RelationMutationIntent::Revalidate(_)) => {}
        }
    }

    (change_count, event_count)
}

/// Apply already sealed input without producing diagnostic/publication copies.
pub(crate) fn apply_original_intents_to_working_state(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    intents: &[crate::transactions::data::MutationIntent],
    version: crate::identity::data::VersionId,
    config: &MutationConfig,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
    symbols: &mut dyn crate::runtime::MutationSymbolMeanings,
    allocations: &mut dyn crate::runtime::MutationRecordAllocations,
) -> Result<(), super::MutationApplyDenial> {
    use crate::transactions::data::TransactionCommitError as Error;
    let mut adjacency = original_adjacency::OriginalAdjacency::default();
    let mut workspace = MutationWorkspace::new(
        *execution,
        state,
        symbols,
        config,
        schema.registry(),
        schema.aspect_plans(),
        version,
        BranchLocalDeleteAllowance::default(),
        Some(allocations),
        None,
    );
    for intent in intents {
        crate::execution::check_request(execution)
            .map_err(|cause| Error::execution(cause.into()))?;
        let outcome = super::intents::dispatch_original_intent(intent, &mut workspace)?;
        for change in outcome.changes {
            super::canonical_deltas::write_original_revision_stamps(
                &change,
                &mut workspace,
                schema,
                version,
            )?;
            for entry in original_adjacency(&change).into_iter().flatten() {
                adjacency.push(execution, entry)?;
            }
        }
    }
    // Preserve ordinary ordering: all row mutations/revisions precede the
    // accumulated endpoint edits. The original cursor must be exhausted too.
    drop(workspace);
    allocations.finish_mutation().map_err(|denial| {
        Error::conflict(crate::transactions::data::CommitConflict::new(
            crate::transactions::data::ConflictClass::RecordAllocationDenied { denial },
        ))
    })?;
    super::record_changes::apply_adjacency_deltas_for_request(
        execution,
        state,
        adjacency.as_slice(),
        version,
    )?;
    Ok(())
}

fn original_adjacency(
    change: &super::outcomes::RecordMutation,
) -> [Option<super::AdjacencyDelta>; 2] {
    use super::outcomes::RecordMutation as Change;
    use super::{AdjacencyDelta as Delta, AdjacencyDeltaKind as Kind};
    let entry = |relation_id, kind_id, source, target, created| {
        Some(Delta {
            relation_id,
            kind_id,
            kind: if created {
                Kind::Created { source, target }
            } else {
                Kind::Deleted { source, target }
            },
        })
    };
    match *change {
        Change::RelationCreated {
            relation_id,
            kind_id,
            source,
            target,
            ..
        }
        | Change::RelationRematerialized {
            relation_id,
            kind_id,
            source,
            target,
            ..
        } => [entry(relation_id, kind_id, source, target, true), None],
        Change::RelationDeleted {
            relation_id,
            kind_id,
            source,
            target,
            ..
        }
        | Change::RelationMaterializationSuspended {
            relation_id,
            kind_id,
            source,
            target,
            ..
        } => [entry(relation_id, kind_id, source, target, false), None],
        Change::RelationUpdated {
            relation_id,
            kind_id,
            old_source,
            old_target,
            new_source,
            new_target,
            ..
        } => [
            entry(relation_id, kind_id, old_source, old_target, false),
            entry(relation_id, kind_id, new_source, new_target, true),
        ],
        _ => [None, None],
    }
}
