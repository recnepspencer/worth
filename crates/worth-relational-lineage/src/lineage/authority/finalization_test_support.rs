use super::posture_finalization::prepare_posture_finalization;
use super::staged_effects::lineage_error;
use crate::contracts::history::data::commit_receipt::RelationalCommitReceipt;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalization;
use crate::contracts::transactions::data::intents::mutation_intent::MutationIntent;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::contracts::transactions::data::primitives::RecordRef;
use crate::runtime::state::subsystems::lineage_identity::LineageIdentityAllocator;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

pub fn finalize_postures_in_working_state_for_test(
    allocator: &LineageIdentityAllocator,
    staged: &mut WorkingState,
    commit: &RelationalCommitReceipt,
    merged_plan: &[MutationIntent],
    changed_records: &[RecordRef],
    input: &super::posture_fixture::LineageCommitPostureInput,
) -> Result<PreparedLineageFinalization, TransactionCommitError> {
    let prepared = prepare_posture_finalization(
        allocator,
        staged,
        commit,
        merged_plan,
        changed_records,
        &input.scheme_registry,
        None,
        input,
        &input.postures,
        None,
    )?;
    install_prepared_bindings(staged, prepared)
}

pub fn finalize_with_schema_for_test(
    allocator: &LineageIdentityAllocator,
    staged: &mut WorkingState,
    commit: &RelationalCommitReceipt,
    changed_records: &[RecordRef],
    schema: &crate::contracts::schema::data::RelationalSchemaRegistry,
) -> Result<PreparedLineageFinalization, TransactionCommitError> {
    let empty = super::posture_fixture::LineageCommitPostureInput::default();
    let prepared = prepare_posture_finalization(
        allocator,
        staged,
        commit,
        &[],
        changed_records,
        schema.persistent_name_schemes(),
        Some(schema),
        &empty,
        &[],
        None,
    )?;
    install_prepared_bindings(staged, prepared)
}

fn install_prepared_bindings(
    staged: &mut WorkingState,
    mut prepared: PreparedLineageFinalization,
) -> Result<PreparedLineageFinalization, TransactionCommitError> {
    for (entity, lineage) in prepared.take_entity_bindings() {
        install_lineage_id_in_working_state(staged, entity, lineage)?;
    }
    Ok(prepared)
}

fn install_lineage_id_in_working_state(
    staged: &mut WorkingState,
    entity_id: EntityId,
    lineage_id: LineageId,
) -> Result<(), TransactionCommitError> {
    let arena = &mut staged
        .get_partition_mut(entity_id.partition_id)
        .entity_arena;
    let physical = arena
        .physical_index(entity_id.slot_index())
        .ok_or_else(|| lineage_error("planned lineage entity slot is unavailable".to_owned()))?;
    let extra = arena.extra.get_mut(physical).ok_or_else(|| {
        lineage_error("planned lineage entity metadata is unavailable".to_owned())
    })?;
    extra.lineage_id = Some(lineage_id);
    Ok(())
}
