//! Apply an admitted immutable artifact, preserving the target's record history.
use crate::contracts::transactions::data::outcomes::commit_conflict::{CommitConflict, CommitConflictOwnerPort as _};
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use super::effect_assembly::assemble_effect;
use super::outcomes::{MutationEvent, MutationOutcome, RecordMutation};
use super::{
    BranchLocalDeleteAllowance, MutationApplyDenial, MutationApplyOutcome, MutationWorkspace,
};
use crate::authority::commit::semantic_plan::retained_entity_payload::PreparedRetainedEntityPayload;
use crate::contracts::config::data::policies::MutationConfig;
use crate::contracts::identity::data::primitives::VersionId;
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::pending_allocations::PendingRecordAllocations;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::PendingSymbolMeanings;
use crate::contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog;
use crate::contracts::schema::data::RelationalSchemaRegistry;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;

pub fn apply(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    selected: &PreparedRetainedEntityPayload,
    version: VersionId,
    config: &MutationConfig,
    schema: &RelationalSchemaRegistry,
    plans: &AspectContractPlanCatalog,
    symbols: &mut PendingSymbolMeanings,
    allocations: &mut PendingRecordAllocations,
    budget: Option<crate::contracts::execution::request_work::RequestWorkBudget>,
) -> Result<MutationApplyOutcome, MutationApplyDenial> {
    let mut workspace = MutationWorkspace::new(
        *execution,
        state,
        symbols,
        config,
        schema,
        plans,
        version,
        BranchLocalDeleteAllowance::default(),
        Some(allocations),
        budget,
    );
    let entity = selected.meaning.entity();
    let kind = selected.meaning.kind();
    let before = workspace.with_context(|context| {
        context
            .state
            .mark_entity_slot_touched_for_request(
                entity.partition_id,
                entity.slot_index(),
                &context.execution,
            )
            .map_err(request_denial)?;
        let partition = context
            .state
            .get_partition_mut_for_request(entity.partition_id, &context.execution)
            .map_err(request_denial)?;
        let old = partition
            .entity_arena
            .get(&entity)
            .filter(|slot| slot.is_live() && slot.kind_id() == Some(kind))
            .ok_or_else(stale)?;
        let before = old.retained_extra();
        let mut extra = selected
            .payload
            .copy_for_request(&context.execution)
            .map_err(request_denial)?;
        extra.lineage_id = before.lineage_id;
        extra.structural_fingerprint = before.structural_fingerprint;
        partition
            .entity_arena
            .apply_extra_update_for_request(&context.execution, entity.slot_index(), extra, version)
            .map_err(request_denial)?
            .ok_or_else(stale)?;
        Ok::<_, crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError>(before)
    })?;
    let after = workspace.with_context(|context| {
        context
            .state
            .retained_entity_extra(entity.partition_id, entity.slot_index())
    });
    let mut outcome = MutationOutcome::with_capacity(execution, 1, 1)?;
    outcome.record_change(
        execution,
        RecordMutation::EntityUpdated {
            entity_id: entity,
            kind_id: kind,
            old_authoritative_aspect_state: Some(before),
            new_authoritative_aspect_state: after,
            authoritative_patch: None,
        },
    )?;
    outcome.record_event(
        execution,
        MutationEvent::EntityUpdated { entity_id: entity },
    )?;
    let effect = assemble_effect(outcome, &mut workspace)?;
    let (created_entities, created_relations) = workspace.into_created_bindings();
    Ok(MutationApplyOutcome {
        effect,
        created_entities,
        created_relations,
    })
}

fn stale() -> CommitConflict {
    CommitConflict::new(ConflictClass::StaleValidationBasis {
        detail: "retained selection target no longer has its admitted full-GEN entity".to_owned(),
    })
}

fn request_denial(
    cause: crate::contracts::execution::denial_cause::RelationalExecutionDenialCause,
) -> crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError
{
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(
        cause.into(),
    )
}
