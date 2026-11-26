//! Install fully prepared checkpoint state into the exclusive recovery runtime.
use super::{
    cold_directory, prepare_checkpoint_state, root_readmission, CheckpointPreparationInputs,
    PreparedCheckpointState,
};
use crate::durability::data::{CheckpointRestoreWork, DurableCheckpoint, RelationalRecoveryStop};
use crate::runtime::RelationalRuntime;
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointOwnerPort as _;

pub(in super::super) fn restore_checkpoint_state(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    checkpoint: DurableCheckpoint,
    selected_checkpoint_acknowledged: bool,
    work: &mut CheckpointRestoreWork,
    original_schemas: Option<&super::super::original_schemas::OriginalSchemaGraph>,
) -> Result<(), RelationalRecoveryStop> {
    if let Some(directory) = checkpoint
        .selected_directory()
        .cloned()
        .map(|edge| edge.into_original())
    {
        let schemas = install_cold_directory(
            lane,
            execution,
            restored,
            &checkpoint,
            &directory,
            selected_checkpoint_acknowledged,
            work,
        )?;
        restored
            .indexes
            .install_cold_checkpoint(lane, directory, restored, schemas)?;
        restored
            .index_authority()
            .recover_installed_index_routing(execution)?;
        super::super::unshared_state(restored)?
            .durability
            .push_checkpoint(checkpoint);
        return Ok(());
    }
    let prepared = prepare_checkpoint_state(
        execution,
        CheckpointPreparationInputs {
            registry: &restored.config.schema.registry,
            aspect_plans: &restored.schema_contract_runtime.aspect_contract_plans,
            symbols: &restored.services.symbols,
            history: &restored.history,
            lineage: &restored.lineage,
        },
        &checkpoint,
        work,
        original_schemas,
    )?;
    let index_roots = install_checkpoint_state(
        super::super::unshared_state(restored)?,
        prepared,
        &checkpoint,
        work,
    )?;
    restored
        .index_authority()
        .recover_installed_index_routing(execution)?;
    if let Some(artifact) = checkpoint.derived_index_checkpoint() {
        crate::indexes::verify_checkpoint_index_bases(
            execution,
            &restored.indexes.snapshot(),
            artifact.bases(),
            &crate::indexes::checkpoint_index_projections(lane, restored),
            &index_roots.roots,
        )?;
    }
    let restored = super::super::unshared_state(restored)?;
    if let Some(directory) = checkpoint
        .original_canonical_directory()
        .cloned()
        .map(|edge| edge.into_original())
    {
        restored
            .durability
            .checkpoint_selection()
            .restore_data(directory);
    }
    restored.durability.push_checkpoint(checkpoint);
    Ok(())
}

fn install_cold_directory(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    checkpoint: &DurableCheckpoint,
    directory: &std::sync::Arc<crate::durability::data::RelationalCheckpointDirectory>,
    selected_checkpoint_acknowledged: bool,
    work: &mut CheckpointRestoreWork,
) -> Result<
    crate::durability::checkpoints::region_directory::ReadmittedSchemas,
    RelationalRecoveryStop,
> {
    crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
    if directory.admission_image.is_some() {
        let owner = crate::durability::checkpoint_admission::CheckpointAdmissionOwner::bind(
            directory, execution,
        )
        .map_err(RelationalRecoveryStop::AdmissionImage)?;
        restored
            .durability
            .checkpoint_selection()
            .install_admission_owner(owner)
            .map_err(RelationalRecoveryStop::AdmissionImage)?;
    }
    let prepared = cold_directory::prepare(
        lane,
        execution,
        &restored.history,
        &restored.services.symbols,
        &restored.config.schema.registry,
        checkpoint,
        directory,
        work,
        restored
            .partitions
            .working_set(restored.config.storage.residency_budget)
            .as_ref(),
        restored.durability.checkpoint_selection(),
    )?;
    let history = restore_canonical_routes(
        &restored.history,
        prepared.history,
        checkpoint,
        Some(directory.manifest.covered_position),
        true,
        work,
    )?;
    let state = super::super::unshared_state(restored)?;
    state.services.symbols = prepared.symbols;
    state.record_identity = prepared.record_identity;
    state.partitions.install_cells(prepared.mirror);
    state.history = history;
    cold_directory::readmit_lineage(
        lane,
        execution,
        &state.lineage,
        checkpoint,
        directory,
        prepared.has_naming_schemes,
        state.history.cold_commit_catalog(),
    )?;
    // Only an acknowledged selected checkpoint becomes the selection owner;
    // an unacknowledged directory restores data without claiming selection.
    if selected_checkpoint_acknowledged {
        state
            .durability
            .checkpoint_selection()
            .restore_selected(directory.clone());
    } else {
        state
            .durability
            .checkpoint_selection()
            .restore_data(directory.clone());
    }
    Ok(prepared.schemas)
}

fn install_checkpoint_state(
    restored: &mut crate::runtime::RelationalRuntimeState,
    prepared: PreparedCheckpointState,
    checkpoint: &DurableCheckpoint,
    work: &mut CheckpointRestoreWork,
) -> Result<root_readmission::ReadmittedRootImages, RelationalRecoveryStop> {
    let floor = prepared.history.latest_recorded_patch_position();
    let history = restore_canonical_routes(
        &restored.history,
        prepared.history,
        checkpoint,
        floor,
        false,
        work,
    )?;
    restored.services.symbols.replace(prepared.symbols);
    restored.record_identity = prepared.record_identity;
    restored.install_rebuilt_partitions(prepared.partitions);
    restored.history = history;
    restored.lineage.install(prepared.lineage);
    restored.indexes.install(prepared.indexes);
    Ok(prepared.index_roots)
}

pub(in super::super) fn clear_recovery_partition_pins(restored: &mut RelationalRuntime) {
    // Native members carry zero runtime pins. The cold directory has admitted
    // no region yet; clearing it must not materialize or mutate its leaves.
    if restored
        .durability
        .latest_checkpoint()
        .is_some_and(|checkpoint| checkpoint.selected_directory().is_some())
    {
        return;
    }
    let mut writer = restored.rebuild_partitions();
    for partition in writer.resident_partitions_mut() {
        partition.entity_arena.clear_all_pins();
        partition.relation_arena.clear_all_pins();
    }
}

pub(in super::super) fn finalize_restored_runtime(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
) -> Result<(), crate::durability::data::RelationalRecoveryStop> {
    restored
        .index_authority()
        .recover_installed_index_routing(execution)?;
    restored.history.rebuild_branch_head_version_index(lane);
    restored.visibility.cache.clear();
    restored
        .visibility_pins()
        .rebuild_branch_head_visibility_residency(lane);
    Ok(())
}

fn restore_canonical_routes(
    current: &crate::runtime::state::subsystems::history::history_canonical_routes::RelationalPublicationHistory,
    branch: crate::runtime::state::subsystems::history::HistorySubsystem,
    checkpoint: &DurableCheckpoint,
    floor: Option<crate::publication::patch::data::PatchStreamPosition>,
    install_catalog: bool,
    work: &mut CheckpointRestoreWork,
) -> Result<crate::runtime::state::subsystems::history::history_canonical_routes::RelationalPublicationHistory, RelationalRecoveryStop>{
    let corrupt = crate::durability::checkpoints::region_directory::corrupt;
    let history = current
        .replacing_recovered_branch(branch)
        .map_err(|denial| corrupt(format!("history snapshot denied: {denial:?}")))?;
    for commit in checkpoint.envelopes() {
        history
            .install_recovered_canonical_route(std::sync::Arc::new(commit.clone()))
            .map_err(|detail| corrupt(detail.to_owned()))?;
        work.history_envelopes_routed += 1;
    }
    if let Some(floor) = floor {
        history.advance_canonical_stream_floor(floor);
    }
    history.restore_branch_transition_coverage(checkpoint.coverage.up_to_branch_transition);
    if install_catalog {
        history
            .canonical_checkpoint_gate()
            .install_checkpoint_catalog(history.checkpoint_catalog_parts());
    }
    Ok(history)
}
