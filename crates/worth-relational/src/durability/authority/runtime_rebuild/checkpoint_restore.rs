use std::sync::Arc;

use crate::durability::data::{DurabilityError, DurabilityMode, DurableCheckpoint};
use crate::history::data::VersionNode;
use crate::runtime::{HistorySubsystem, IndexingState, LineageState, RelationalRuntime};

use super::super::super::derived_index_artifacts::restore_checkpoint_derived_index_artifacts;
use super::super::authority_continuity::validate_checkpoint_lineage_artifact;

mod branch_root_images;
mod partition_images;
mod record_identity;
mod root_schema_readmission;

use branch_root_images::restore_branch_root_images;
use partition_images::restore_unique_partition_images;
use record_identity::prepare_record_identity;

struct PreparedCheckpointState {
    symbols: crate::symbols::data::StringInterner,
    record_identity: crate::runtime::RecordIdentitySubsystem,
    partitions: std::collections::BTreeMap<
        crate::identity::data::PartitionId,
        crate::storage::overlay::PartitionState,
    >,
    history: HistorySubsystem,
    lineage: LineageState,
    indexes: IndexingState,
    checkpoint: DurableCheckpoint,
}

pub(super) fn restore_checkpoint_state(
    restored: &mut RelationalRuntime,
    checkpoint: &DurableCheckpoint,
) -> Result<(), DurabilityError> {
    let prepared = prepare_checkpoint_state(restored, checkpoint)?;
    install_checkpoint_state(super::unshared_state(restored)?, prepared);
    Ok(())
}

fn prepare_checkpoint_state(
    restored: &mut RelationalRuntime,
    checkpoint: &DurableCheckpoint,
) -> Result<PreparedCheckpointState, DurabilityError> {
    validate_checkpoint_lineage_artifact(checkpoint)?;
    let symbols = prepare_symbols(restored, checkpoint);
    let record_identity = prepare_record_identity(checkpoint)?;
    let branch_root_images = restore_branch_root_images(restored, checkpoint)?;
    let partitions = prepare_partitions(restored, checkpoint)?;
    let history = prepare_history(restored, checkpoint, &branch_root_images, &symbols)?;
    Ok(PreparedCheckpointState {
        symbols,
        record_identity,
        partitions,
        history,
        lineage: prepare_lineage(restored, checkpoint),
        indexes: prepare_indexes(checkpoint),
        checkpoint: checkpoint.clone(),
    })
}

fn prepare_symbols(
    restored: &mut RelationalRuntime,
    checkpoint: &DurableCheckpoint,
) -> crate::symbols::data::StringInterner {
    let mut symbols = restored.services.symbols.interner_snapshot();
    symbols.restore_snapshot(checkpoint.symbol_table.clone());
    symbols
}

fn prepare_partitions(
    restored: &mut RelationalRuntime,
    checkpoint: &DurableCheckpoint,
) -> Result<
    std::collections::BTreeMap<
        crate::identity::data::PartitionId,
        crate::storage::overlay::PartitionState,
    >,
    DurabilityError,
> {
    let aspect_contracts = crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog::readmit(
        &checkpoint.aspect_contracts,
    )?;
    let mut partitions = restore_unique_partition_images(
        restored,
        &checkpoint.partition_images,
        &aspect_contracts,
        "checkpoint",
    )?;
    crate::storage::partition::rebuild_adjacency_kind_buckets(&mut partitions).map_err(
        |detail| {
            DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                detail,
            )
        },
    )?;
    Ok(partitions)
}

fn prepare_history(
    restored: &mut RelationalRuntime,
    checkpoint: &DurableCheckpoint,
    branch_roots: &branch_root_images::RestoredBranchRootImages,
    symbols: &crate::symbols::data::StringInterner,
) -> Result<HistorySubsystem, DurabilityError> {
    let mut history = restored.history.detached_owner_snapshot();
    history.with_ledger_mut(|ledger| {
        ledger.commit_envelopes = checkpoint
            .envelopes
            .iter()
            .map(|commit| {
                (
                    commit.envelope().commit.commit_id,
                    Arc::clone(commit.canonical_arc()),
                )
            })
            .collect();
        ledger.patch_stream_index = checkpoint
            .envelopes
            .iter()
            .map(|commit| (commit.position(), commit.envelope().commit.commit_id))
            .collect();
    });
    for commit in &checkpoint.envelopes {
        history
            .install_recovered_canonical_route(Arc::new(commit.clone()))
            .map_err(|detail| {
                DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                    detail,
                )
            })?;
    }
    if let Some(position) = history.latest_recorded_patch_position() {
        history.advance_canonical_stream_floor(position);
    }
    let commit_graph = checkpoint
        .envelopes
        .iter()
        .map(|envelope| {
            (
                envelope.commit.commit_id,
                VersionNode {
                    commit: envelope.commit.clone(),
                },
            )
        })
        .collect();
    history.with_ledger_mut(|ledger| ledger.commit_graph = commit_graph);
    history.rebuild_catalog_from_durable_envelopes();
    history
        .restore_branch_cells(
            &checkpoint.branch_cells,
            &branch_roots.partitions,
            &branch_roots.schema_authorities,
            &restored.config.schema.registry,
            symbols,
        )
        .map_err(|detail| {
            DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                detail,
            )
        })?;
    Ok(history)
}

fn prepare_lineage(
    restored: &mut RelationalRuntime,
    checkpoint: &DurableCheckpoint,
) -> LineageState {
    let mut lineage = restored.lineage.snapshot();
    lineage.nodes = checkpoint
        .lineage
        .nodes()
        .iter()
        .cloned()
        .map(|node| (node.lineage_id, node))
        .collect();
    let events = checkpoint
        .envelopes
        .iter()
        .flat_map(|envelope| {
            envelope
                .lineage_events()
                .iter()
                .cloned()
                .map(|event| (event, envelope.commit.commit_id))
        })
        .collect::<Vec<_>>();
    let mut events = events;
    events.sort_by_key(|(event, _)| event.event_id);
    lineage.replace_events(events);
    lineage
}

fn prepare_indexes(checkpoint: &DurableCheckpoint) -> IndexingState {
    let mut indexes = IndexingState::default();
    for definition in &checkpoint.index_definitions {
        indexes.insert_definition(definition.clone());
    }
    restore_checkpoint_derived_index_artifacts(&mut indexes, &checkpoint.derived_index_artifacts);
    indexes
}

fn install_checkpoint_state(
    restored: &mut crate::runtime::RelationalRuntimeState,
    prepared: PreparedCheckpointState,
) {
    restored.services.symbols.replace(prepared.symbols);
    restored.record_identity = prepared.record_identity;
    restored.install_rebuilt_partitions(prepared.partitions);
    restored.history = prepared.history;
    restored.lineage.install(prepared.lineage);
    restored.indexes.install(prepared.indexes);
    restored.durability.push_checkpoint(prepared.checkpoint);
}

pub(super) fn clear_recovery_partition_pins(restored: &mut RelationalRuntime) {
    let mut writer = restored.rebuild_partitions();
    for partition in writer.partitions_mut() {
        partition.entity_arena.clear_all_pins();
        partition.relation_arena.clear_all_pins();
    }
}

pub(super) fn finalize_restored_runtime(
    restored: &mut RelationalRuntime,
    original_durability_mode: DurabilityMode,
) -> Result<(), crate::durability::data::DurabilityError> {
    restored.set_durability_mode(original_durability_mode);
    restored
        .index_authority()
        .rebuild_unique_entity_aspect_field_indexes()
        .map_err(|denial| {
            crate::durability::data::DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::ReplayFailure,
                format!("recovered unique-index basis admission failed: {denial:?}"),
            )
        })?;
    restored.history.rebuild_branch_head_version_index();
    restored.visibility.cache.clear();
    restored
        .visibility_pins()
        .rebuild_branch_head_visibility_residency();
    Ok(())
}
