use super::posture_finalization::prepare_posture_finalization;
use super::staged_effects::lineage_error;
use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::PreparedLineageFinalization;
use crate::runtime::state::subsystems::lineage_identity::LineageIdentityAllocator;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::{
    intents::MutationIntent, outcomes::transaction_commit_error::TransactionCommitError,
    primitives::RecordRef,
};
use worth_relational_contracts::internal_port::indexes::data::entry_map::DerivedIndexEntryMapOwnerPort as _;

pub(crate) fn finalize_postures_in_working_state(
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

pub(crate) fn test_finalize_with_schema(
    allocator: &LineageIdentityAllocator,
    staged: &mut WorkingState,
    commit: &RelationalCommitReceipt,
    changed_records: &[RecordRef],
    schema: &crate::schema::data::RelationalSchemaRegistry,
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
