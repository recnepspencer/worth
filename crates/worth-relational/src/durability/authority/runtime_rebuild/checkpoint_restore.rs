use crate::durability::data::{
    CheckpointRestoreWork, DurabilityError, DurableCheckpoint, RelationalRecoveryStop,
};
use crate::runtime::state::subsystems::{
    history::HistorySubsystem,
    indexing::IndexingState,
    lineage::{LineageState, LineageSubsystem},
    record_identity::RecordIdentitySubsystem,
};
use worth_relational_contracts::internal_port::lineage::data::events::LineageEventRecordFieldsPort as _;
use worth_relational_contracts::internal_port::lineage::data::graph::LineageNodeFieldsPort as _;
use worth_relational_contracts::internal_port::symbols::data::string_interner::StringInternerOwnerPort as _;

use super::super::super::derived_index_artifacts::restore_checkpoint_derived_index_artifacts;
use super::super::authority_continuity::validate_checkpoint_lineage_artifact;

mod branch_root_images;
mod cold_directory;
mod history_readmission;
pub(super) mod installation;
mod selected_catalog;
use history_readmission::prepare_history;
mod partition_images;
mod record_identity;
mod recovered_codec_meaning;
mod recovered_relation_laws;
mod recovered_schema_meaning;
mod root_readmission;
mod root_schema_readmission;

#[cfg(test)]
#[path = "checkpoint_restore_tests.rs"]
mod tests;

use branch_root_images::restore_branch_root_images;
use record_identity::prepare_record_identity;

pub(super) struct PreparedCheckpointState {
    symbols: crate::symbols::data::StringInterner,
    record_identity: RecordIdentitySubsystem,
    partitions: std::collections::BTreeMap<
        crate::identity::data::PartitionId,
        crate::storage::overlay::PartitionState,
    >,
    history: HistorySubsystem,
    lineage: LineageState,
    indexes: IndexingState,
    index_roots: root_readmission::ReadmittedRootImages,
}

struct CheckpointPreparationInputs<'a> {
    registry: &'a crate::schema::data::RelationalSchemaRegistry,
    aspect_plans: &'a crate::schema::data::AspectContractPlanCatalog,
    symbols: &'a crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    history: &'a HistorySubsystem,
    lineage: &'a LineageSubsystem,
}

fn prepare_checkpoint_state(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: CheckpointPreparationInputs,
    checkpoint: &DurableCheckpoint,
    work: &mut CheckpointRestoreWork,
    original_schemas: Option<&super::original_schemas::OriginalSchemaGraph>,
) -> Result<PreparedCheckpointState, RelationalRecoveryStop> {
    validate_checkpoint_lineage_artifact(checkpoint)?;
    let symbols =
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            prepare_symbols(lane, inputs.symbols.interner_snapshot(), checkpoint)
        })?;
    let record_identity = prepare_record_identity(checkpoint)?;
    let branch_root_images =
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            restore_branch_root_images(
                lane,
                execution,
                inputs.registry,
                inputs.aspect_plans,
                checkpoint,
                work,
                original_schemas,
            )
        })?;
    recovered_schema_meaning::check_current_schema_meaning(
        execution,
        checkpoint,
        &branch_root_images,
        inputs.registry,
    )?;
    let partitions = prepare_partitions(
        execution,
        inputs.aspect_plans,
        checkpoint,
        &branch_root_images,
        work,
    )?;
    let prepared_history =
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(
            |_lane| {
                prepare_history(
                    execution,
                    inputs.history,
                    inputs.registry,
                    checkpoint,
                    branch_root_images,
                    &symbols,
                    work,
                )
            },
        )?;
    let lineage =
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            prepare_lineage(lane, inputs.lineage.snapshot(), checkpoint)
        });
    let indexes =
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            prepare_indexes(lane, checkpoint, &prepared_history.history, work)
        })?;
    Ok(PreparedCheckpointState {
        symbols,
        record_identity,
        partitions,
        history: prepared_history.history,
        index_roots: prepared_history.roots,
        lineage,
        indexes,
    })
}

fn prepare_symbols(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    mut symbols: crate::symbols::data::StringInterner,
    checkpoint: &DurableCheckpoint,
) -> Result<crate::symbols::data::StringInterner, DurabilityError> {
    symbols.restore_snapshot(lane, checkpoint.symbol_table.clone());
    symbols
        .reserve_frontier(checkpoint.symbol_identity_frontier)
        .map_err(|detail| {
            DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                detail,
            )
        })?;
    Ok(symbols)
}

fn prepare_partitions(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    aspect_plans: &crate::schema::data::AspectContractPlanCatalog,
    checkpoint: &DurableCheckpoint,
    branch_roots: &branch_root_images::RestoredBranchRootImages,
    work: &mut CheckpointRestoreWork,
) -> Result<
    std::collections::BTreeMap<
        crate::identity::data::PartitionId,
        crate::storage::overlay::PartitionState,
    >,
    RelationalRecoveryStop,
> {
    let aspect_contracts = crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog::readmit(
        &checkpoint.aspect_contracts,
    )?;
    crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
        partition_images::restore_partition_mirror(
            lane,
            execution,
            partition_images::CheckpointMirrorImages {
                partitions: &checkpoint.partition_images,
                kinds: &checkpoint.mirror_kind_images,
            },
            checkpoint
                .branch_roots
                .iter()
                .map(|root| partition_images::CheckpointRootPartitions {
                    commit_id: root.commit_id,
                    partitions: &root.partition_images,
                }),
            branch_roots,
            aspect_plans,
            &aspect_contracts,
            work,
        )
    })
}

fn prepare_lineage(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    mut lineage: LineageState,
    checkpoint: &DurableCheckpoint,
) -> LineageState {
    lineage.nodes = checkpoint
        .lineage
        .nodes()
        .iter()
        .cloned()
        .map(|node| ((*node.owner_lineage_id()), node))
        .collect();
    for cell in &checkpoint.branch_cells {
        if let (Some(source_branch), Some(provenance)) =
            (&cell.fork_source_branch_id, &cell.fork_provenance)
            && let worth_foundational::FoundationalBranchTarget::Basis(target) = provenance.target()
        {
            lineage.record_fork_ancestry_metadata(
                cell.branch_id.clone(),
                source_branch.clone(),
                crate::history::data::CommitId(target.selected_commit_id()),
            );
        }
    }
    for &(relation_id, lineage_id) in checkpoint.lineage.relation_lineages() {
        lineage.record_relation_lineage(relation_id, lineage_id);
    }
    for envelope in &checkpoint.envelopes {
        for &(relation_id, lineage_id) in envelope.envelope().lineage_relation_lineages() {
            lineage.record_relation_lineage(relation_id, lineage_id);
        }
    }
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
    events.sort_by_key(|(event, _)| (*event.owner_event_id()));
    lineage.replace_events(lane, events);
    lineage
}

fn prepare_indexes(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    checkpoint: &DurableCheckpoint,
    history: &HistorySubsystem,
    work: &mut CheckpointRestoreWork,
) -> Result<IndexingState, DurabilityError> {
    if !crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::supports_outer_format(
        checkpoint.derived_index_checkpoint_format,
        checkpoint.derived_index_checkpoint.is_some(),
    ) {
        return Err(DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
            "derived index checkpoint format or payload is missing",
        ));
    }
    let mut indexes = IndexingState::default();
    indexes.restore_generation_frontier(checkpoint.index_generation_frontier);
    indexes
        .readmit_checkpoint_catalog(
            &checkpoint.index_definitions,
            checkpoint.index_definition_frontier,
        )
        .map_err(|detail| {
            DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                detail,
            )
        })?;
    work.index_definitions_readmitted += checkpoint.index_definitions.len();
    restore_checkpoint_derived_index_artifacts(
        lane,
        &mut indexes,
        history,
        &checkpoint.derived_index_artifacts,
        checkpoint.derived_index_checkpoint.as_ref(),
        checkpoint.index_generation_frontier,
        work,
    )?;
    Ok(indexes)
}

pub(super) fn shared_rebuild_refusal() -> DurabilityError {
    DurabilityError::new(
        crate::durability::data::RecoveryFailureClass::ReplayFailure,
        "recovery cannot replace the subsystems of a runtime already bound to a service",
    )
}
