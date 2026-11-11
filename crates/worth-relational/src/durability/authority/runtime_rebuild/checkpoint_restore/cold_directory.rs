//! Install immutable leaves and indexed history. No member is read here.
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::durability::checkpoints::region_directory::{corrupt, ReadmittedRegionDirectory};
use crate::durability::data::{CheckpointRestoreWork, DurableCheckpoint, RelationalRecoveryStop};
use crate::history::data::VersionNode;
use crate::runtime::RelationalRuntime;
use crate::storage::residency::RelationalRegionRetention;

pub(super) fn restore(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    checkpoint: &DurableCheckpoint,
    directory: Arc<crate::durability::data::RelationalCheckpointDirectory>,
    work: &mut CheckpointRestoreWork,
) -> Result<
    crate::durability::checkpoints::region_directory::ReadmittedSchemas,
    RelationalRecoveryStop,
> {
    crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
    if directory.admission_image.is_some() {
        let owner = crate::durability::checkpoint_admission::CheckpointAdmissionOwner::bind(
            &directory, execution,
        )
        .map_err(RelationalRecoveryStop::AdmissionImage)?;
        runtime
            .durability
            .checkpoint_selection()
            .install_admission_owner(owner)
            .map_err(RelationalRecoveryStop::AdmissionImage)?;
    }
    let mut history = runtime
        .history
        .detached_owner_snapshot()
        .map_err(|denial| corrupt(format!("cold history snapshot denied: {denial:?}")))?;
    let mut symbols = runtime.services.symbols.interner_snapshot();
    symbols.restore_snapshot(lane, checkpoint.symbol_table.clone());
    let paged_symbols = if directory.admission_image.is_some() {
        runtime
            .services
            .symbols
            .from_selected_directory_for_request(&directory, execution)?
    } else {
        let paged = runtime.services.symbols.detached_owner_snapshot();
        paged.replace(symbols);
        paged
            .install_cold_pages(&directory)
            .map_err(crate::runtime::ColdSymbolDirectoryDenial::into_recovery_stop)?;
        paged
    };
    let mut regions = ReadmittedRegionDirectory::new(
        directory.clone(),
        paged_symbols.clone(),
        execution,
        &runtime.config.schema.registry.content_formats,
    )?;
    let mut head_ids = worth_execution::ExecutionOrderedMap::default();
    for cell in &checkpoint.branch_cells {
        crate::durability::artifact_payload_emission::request::account_work(execution, 1)?;
        if let worth_foundational::FoundationalBranchTarget::Basis(target) =
            cell.observation.target()
        {
            let id = crate::history::data::CommitId(target.selected_commit_id());
            if !head_ids.contains_key(&id) {
                head_ids
                    .insert_cloned_native_for_request(&id, &(), *execution)
                    .map_err(crate::durability::checkpoint_history::capture::map_stop)?;
            }
        }
    }
    let positioned = checkpoint
        .envelopes
        .iter()
        .map(|entry| (entry.commit.commit_id, entry.clone()))
        .collect::<BTreeMap<_, _>>();
    let catalog = super::selected_catalog::admit(execution, &directory, |entry| {
        positioned.get(&entry.commit.commit_id).cloned()
    })?;
    let head_cells = head_ids
        .iter()
        .map(|(id, _)| {
            catalog
                .get(*id)
                .map(|artifact| (*id, artifact.envelope_residency()))
                .ok_or_else(|| corrupt("admitted head has no catalog entry"))
        })
        .collect::<Result<_, _>>()?;
    history.with_ledger_mut(|ledger| {
        ledger.commit_catalog = catalog;
        ledger.commit_envelopes = head_cells;
        ledger.commit_graph = directory
            .manifest
            .commits
            .iter()
            .map(|entry| {
                (
                    entry.commit.commit_id,
                    VersionNode {
                        commit: entry.commit.clone(),
                    },
                )
            })
            .collect();
        ledger.patch_stream_index = directory
            .manifest
            .commits
            .iter()
            .map(|entry| (entry.position, entry.commit.commit_id))
            .collect();
    });
    for commit in &checkpoint.envelopes {
        history
            .install_recovered_canonical_route(Arc::new(commit.clone()))
            .map_err(corrupt)?;
        work.history_envelopes_routed += 1;
    }
    history.advance_canonical_stream_floor(directory.manifest.covered_position);
    let mut roots = BTreeMap::new();
    let mut authorities = BTreeMap::new();
    for image in directory
        .manifest
        .roots
        .iter()
        .filter(|image| head_ids.contains_key(&image.commit_id))
    {
        let schema = regions
            .schemas
            .get(&image.schema_carrier_digest)
            .cloned()
            .ok_or_else(|| corrupt("root has no schema carrier"))?;
        let root = regions.readmit_root(&history, image, execution)?;
        roots.insert(image.commit_id, root);
        authorities.insert(image.commit_id, schema);
        work.root_images_verified += 1;
    }
    super::recovered_schema_meaning::check_current_schema_meaning(
        execution,
        checkpoint,
        &super::branch_root_images::RestoredBranchRootImages {
            partitions: BTreeMap::new(),
            schema_authorities: authorities,
        },
        &runtime.config.schema.registry,
    )?;
    history
        .restore_branch_cells(
            &checkpoint.branch_cells,
            &checkpoint.branch_lifecycle,
            &checkpoint.retired_branch_names,
            &roots,
            &runtime.config.schema.registry,
        )
        .map_err(corrupt)?;
    history.restore_branch_transition_coverage(checkpoint.coverage.up_to_branch_transition);
    history
        .canonical_checkpoint_gate()
        .install_checkpoint_catalog(history.commit_artifacts());
    work.branch_cells_readmitted += checkpoint.branch_cells.len();
    let mirror = match directory.manifest.mirror_schema {
        Some(schema) => regions
            .regions(schema, &directory.manifest.mirror, execution)?
            .iter()
            .cloned()
            .map(|region| {
                (
                    region.partition_id,
                    RelationalRegionRetention::new(region.cell, region.kind_authority),
                )
            })
            .collect(),
        None if directory.manifest.mirror.is_empty() => Default::default(),
        None => return Err(corrupt("mirror has no schema carrier").into()),
    };
    let record_identity = super::record_identity::prepare_record_identity(checkpoint)?;
    let runtime = super::super::unshared_state(runtime)?;
    runtime.services.symbols = paged_symbols;
    runtime.record_identity = record_identity;
    runtime.partitions.install_cells(mirror);
    runtime.history = history;
    runtime.lineage.install_cold_pages(&directory)?;
    runtime
        .durability
        .checkpoint_selection()
        .restore_selected(directory);
    Ok(regions.schemas)
}
