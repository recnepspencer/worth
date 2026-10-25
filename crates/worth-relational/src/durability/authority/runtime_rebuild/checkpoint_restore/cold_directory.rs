//! Install immutable leaves and indexed history. No member is read here.
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::durability::checkpoints::region_directory::{corrupt, ReadmittedRegionDirectory};
use crate::durability::data::{CheckpointRestoreWork, DurableCheckpoint, RelationalRecoveryStop};
use crate::history::data::VersionNode;
use crate::history::RelationalCommitCatalog;
use crate::runtime::RelationalRuntime;
use crate::storage::residency::RelationalRegionRetention;

pub(super) fn restore(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    checkpoint: &DurableCheckpoint,
    work: &mut CheckpointRestoreWork,
) -> Result<(), RelationalRecoveryStop> {
    crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
    let directory = checkpoint
        .directory
        .as_ref()
        .expect("cold restore has a directory")
        .clone();
    let mut history = runtime.history.detached_owner_snapshot();
    let mut symbols = runtime.services.symbols.interner_snapshot();
    symbols.restore_snapshot(checkpoint.symbol_table.clone());
    let paged_symbols = runtime.services.symbols.detached_owner_snapshot();
    paged_symbols.replace(symbols);
    paged_symbols
        .install_cold_pages(&directory)
        .map_err(corrupt)?;
    let mut regions = ReadmittedRegionDirectory::new(directory.clone(), paged_symbols.clone())?;
    let heads = checkpoint
        .envelopes
        .iter()
        .map(|entry| (entry.commit.commit_id, Arc::clone(entry.canonical_arc())))
        .collect::<BTreeMap<_, _>>();
    let mut catalog = RelationalCommitCatalog::default();
    for entry in &directory.manifest.commits {
        catalog
            .append_indexed_recovery(
                entry.clone(),
                directory.source.clone(),
                heads.get(&entry.commit.commit_id).cloned(),
            )
            .map_err(|denial| corrupt(format!("indexed commit admission: {denial:?}")))?;
    }
    history.with_ledger_mut(|ledger| {
        ledger.commit_catalog = catalog;
        ledger.commit_envelopes = heads.clone();
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
    for image in &directory.manifest.roots {
        let envelope = heads
            .get(&image.commit_id)
            .cloned()
            .ok_or_else(|| corrupt("root has no admitted head envelope"))?;
        let schema = regions
            .schemas
            .get(&image.schema_carrier_digest)
            .cloned()
            .ok_or_else(|| corrupt("root has no schema carrier"))?;
        let descriptor = checkpoint
            .commit_descriptors
            .get(&image.commit_id)
            .cloned()
            .ok_or_else(|| corrupt("root has no commit descriptor"))?;
        let leaves = regions.regions(image.schema_carrier_digest, &image.regions)?;
        let root = history
            .readmit_branch_region_directory(leaves, envelope, descriptor, schema.clone())
            .map_err(|denial| {
                corrupt(format!("branch region directory readmission: {denial:?}"))
            })?;
        roots.insert(image.commit_id, root);
        authorities.insert(image.commit_id, schema);
        work.root_images_verified += 1;
    }
    super::recovered_codec_meaning::check_current_codec_meaning(
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
            .regions(schema, &directory.manifest.mirror)?
            .into_iter()
            .map(|region| {
                (
                    region.partition_id,
                    RelationalRegionRetention::new(region.cell, region.kind_authority),
                )
            })
            .collect(),
        None if directory.manifest.mirror.is_empty() => BTreeMap::new(),
        None => return Err(corrupt("mirror has no schema carrier").into()),
    };
    let record_identity = super::record_identity::prepare_record_identity(checkpoint)?;
    let runtime = super::super::unshared_state(runtime)?;
    runtime.services.symbols = paged_symbols;
    runtime.record_identity = record_identity;
    runtime.partitions.install_cells(mirror);
    runtime.history = history;
    runtime
        .indexes
        .install_cold_checkpoint(directory.clone(), &runtime.history);
    runtime.durability.checkpoint_selection().restore(directory);
    Ok(())
}
