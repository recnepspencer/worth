use crate::durability::data::CheckpointRestoreWork;
use crate::durability::data::DurableCheckpoint;
use contracts::durability::data::recovery_errors::DurabilityError;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::lineage::data::events::LineageEventRecordFieldsPort as _;
use contracts::lineage::data::graph::LineageNodeFieldsPort as _;
use contracts::symbols::data::string_interner::StringInternerOwnerPort as _;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::IndexingState;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::state::LineageState;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::LineageSubsystem;
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::RecordIdentitySubsystem;

use super::super::super::derived_index_artifacts::restore_checkpoint_derived_index_artifacts;
use super::super::authority_continuity::validate_checkpoint_lineage_artifact;

pub mod branch_root_images;
pub mod cold_commits;
pub mod cold_directory;
pub mod history_readmission;

pub mod selected_catalog;
use history_readmission::prepare_history;
pub mod partition_images;
pub mod record_identity;
pub mod recovered_codec_meaning;
pub mod recovered_relation_laws;
pub mod recovered_schema_meaning;
pub mod root_readmission;
pub mod root_schema_readmission;

#[cfg(test)]
#[path = "checkpoint_restore_tests.rs"]
pub mod tests;

use branch_root_images::restore_branch_root_images;
use record_identity::prepare_record_identity;

pub struct PreparedCheckpointState {
    pub symbols: contracts::symbols::data::string_interner::StringInterner,
    pub record_identity: RecordIdentitySubsystem,
    pub partitions: std::collections::BTreeMap<
        contracts::identity::data::primitives::PartitionId,
        worth_relational_storage::internal_port::storage::overlay::partition::PartitionState,
    >,
    pub history: HistorySubsystem,
    pub lineage: LineageState,
    pub indexes: IndexingState,
    pub index_roots: root_readmission::ReadmittedRootImages,
}

pub struct CheckpointPreparationInputs<'a> {
    pub registry: &'a contracts::schema::data::RelationalSchemaRegistry,
    pub aspect_plans: &'a contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog,
    pub symbols: &'a worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    pub history: &'a HistorySubsystem,
    pub lineage: &'a LineageSubsystem,
}

pub fn prepare_checkpoint_state(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: CheckpointPreparationInputs,
    checkpoint: &DurableCheckpoint,
    work: &mut CheckpointRestoreWork,
    original_schemas: Option<&super::original_schemas::OriginalSchemaGraph>,
) -> Result<PreparedCheckpointState, RelationalRecoveryStop> {
    validate_checkpoint_lineage_artifact(checkpoint)?;
    let symbols =
        contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
            prepare_symbols(lane, inputs.symbols.interner_snapshot(), checkpoint)
        })?;
    let record_identity = prepare_record_identity(checkpoint)?;
    let branch_root_images =
        contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
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
        contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(
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
        contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
            prepare_lineage(lane, inputs.lineage.snapshot(), checkpoint)
        })?;
    let indexes =
        contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
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
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    mut symbols: contracts::symbols::data::string_interner::StringInterner,
    checkpoint: &DurableCheckpoint,
) -> Result<contracts::symbols::data::string_interner::StringInterner, DurabilityError> {
    symbols.restore_snapshot(lane, checkpoint.symbol_table.clone());
    symbols
        .reserve_frontier(checkpoint.symbol_identity_frontier)
        .map_err(|detail| {
            DurabilityError::new(
                contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
                detail,
            )
        })?;
    Ok(symbols)
}

fn prepare_partitions(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    aspect_plans: &contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog,
    checkpoint: &DurableCheckpoint,
    branch_roots: &branch_root_images::RestoredBranchRootImages,
    work: &mut CheckpointRestoreWork,
) -> Result<
    std::collections::BTreeMap<
        contracts::identity::data::primitives::PartitionId,
        worth_relational_storage::internal_port::storage::overlay::partition::PartitionState,
    >,
    RelationalRecoveryStop,
> {
    let aspect_contracts = crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog::readmit(
        &checkpoint.aspect_contracts,
    )?;
    contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
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
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    mut lineage: LineageState,
    checkpoint: &DurableCheckpoint,
) -> Result<LineageState, DurabilityError> {
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
                contracts::history::data::coordinates::CommitId(target.selected_commit_id()),
            );
        }
    }
    super::recovered_group_lineage::install_recovered_group_lineage(
        checkpoint
            .envelopes
            .iter()
            .map(|positioned| positioned.envelope()),
        |records| lineage.replace_group_lineage(records),
    )?;
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
    events.sort_by_key(|(event, _)| *event.owner_event_id());
    lineage.replace_events(lane, events);
    Ok(lineage)
}

fn prepare_indexes(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    checkpoint: &DurableCheckpoint,
    history: &HistorySubsystem,
    work: &mut CheckpointRestoreWork,
) -> Result<IndexingState, DurabilityError> {
    if !crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::supports_outer_format(
        checkpoint.derived_index_checkpoint_format,
        checkpoint.derived_index_checkpoint.is_some(),
    ) {
        return Err(DurabilityError::new(
            contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
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
                contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
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

pub fn shared_rebuild_refusal() -> DurabilityError {
    DurabilityError::new(
        contracts::durability::data::recovery_errors::RecoveryFailureClass::ReplayFailure,
        "recovery cannot replace the subsystems of a runtime already bound to a service",
    )
}
