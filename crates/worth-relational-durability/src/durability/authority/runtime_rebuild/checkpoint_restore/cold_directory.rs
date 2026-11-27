//! Admit immutable leaves and indexed history without reading any member.
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;

use crate::durability::checkpoints::region_directory::{corrupt, ReadmittedRegionDirectory};
use crate::durability::data::CheckpointRestoreWork;
use crate::durability::data::DurableCheckpoint;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::schema::data::RelationalSchemaRegistry;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::LineageSubsystem;
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::RecordIdentitySubsystem;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use worth_relational_storage::internal_port::storage::residency::retention::RelationalRegionRetention;

pub struct PreparedColdDirectory {
    pub history: HistorySubsystem,
    pub symbols: RuntimeSymbolTable,
    pub record_identity: RecordIdentitySubsystem,
    pub mirror: worth_relational_storage::internal_port::runtime::state::subsystems::storage::partition_edition::PartitionMap,
    pub schemas: crate::durability::checkpoints::region_directory::ReadmittedSchemas,
    pub has_naming_schemes: bool,
}

pub fn prepare(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source_history: &HistorySubsystem,
    source_symbols: &RuntimeSymbolTable,
    registry: &RelationalSchemaRegistry,
    checkpoint: &DurableCheckpoint,
    directory: &Arc<crate::durability::data::RelationalCheckpointDirectory>,
    work: &mut CheckpointRestoreWork,
    working_set: Option<&Arc<worth_relational_storage::internal_port::storage::residency::working_set::WorkingSetAccountant>>,
    selection: &crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection,
    original_schemas: Option<Arc<super::super::original_schemas::OriginalSchemaGraph>>,
) -> Result<PreparedColdDirectory, RelationalRecoveryStop> {
    let mut history = source_history
        .detached_owner_snapshot()
        .map_err(|denial| corrupt(format!("cold history snapshot denied: {denial:?}")))?;
    let mut symbols = source_symbols.interner_snapshot();
    symbols.restore_snapshot(lane, checkpoint.symbol_table.clone());
    let paged_symbols = if directory.admission_image.is_some() {
        source_symbols.from_selected_directory_for_request(
            directory.manifest.symbol_page_roster(),
            directory.symbol_member_custody(),
            execution,
        )?
    } else {
        let paged = source_symbols.detached_owner_snapshot();
        paged.replace(symbols);
        paged.install_cold_symbol_pages(
            directory.manifest.symbol_page_roster(),
            directory.source.clone(),
        )?;
        paged
    };
    // Branch heads and the mirror serve current reads: the Head pool.
    let mut regions = ReadmittedRegionDirectory::new(
        directory.clone(),
        paged_symbols.clone(),
        execution,
        registry,
    )?
    .with_working_set(working_set.map(|accountant| {
        (
            accountant.clone(),
            worth_relational_storage::internal_port::storage::residency::working_set::ResidencyPoolKind::Head,
        )
    }));
    let mut head_ids = worth_execution::ExecutionOrderedMap::default();
    for cell in &checkpoint.branch_cells {
        contracts::durability::data::recovery_stop::account_work(execution, 1)?;
        if let worth_foundational::FoundationalBranchTarget::Basis(target) =
            cell.observation.target()
        {
            let id = contracts::history::data::coordinates::CommitId(target.selected_commit_id());
            if !head_ids.contains_key(&id) {
                head_ids
                    .insert_cloned_native_for_request(&id, &(), *execution)
                    .map_err(contracts::durability::data::recovery_stop::map_stop)?;
            }
        }
    }
    // G7: no artifact, graph node or patch position is installed per commit.
    // Only the selected heads become resident (their entries are inline in
    // the commit index); every other commit faults through the cold catalog.
    let positioned = checkpoint
        .envelopes
        .iter()
        .map(|entry| (entry.commit.commit_id, entry))
        .collect::<BTreeMap<_, _>>();
    let cold = worth_relational_branch::internal_port::history::commit::cold_catalog::ColdCommitCatalog::new(
        Arc::new(super::cold_commits::SelectedCommitSource::new(
            selection.clone(),
            directory,
            working_set.cloned(),
            original_schemas,
        )),
    );
    selection.register_cold_catalog(&cold);
    let mut catalog =
        worth_relational_branch::internal_port::history::commit::catalog::RelationalCommitCatalog::default();
    let mut head_cells = BTreeMap::new();
    for (id, _) in head_ids.iter() {
        let artifact = cold
            .artifact(*id)
            .map_err(|cause| crate::durability::data::commit_index_access::stop(0, cause))?
            .ok_or_else(|| corrupt("admitted head has no catalog entry"))?;
        if let Some(admitted) = positioned.get(id) {
            artifact
                .envelope_residency()
                .bind_recovered_positioned(execution, admitted)?;
        }
        head_cells.insert(*id, artifact.envelope_residency());
        catalog
            .append_preencoded_recovery(artifact)
            .map_err(|cause| corrupt(format!("selected head insertion: {cause:?}")))?;
    }
    history.with_ledger_mut(|ledger| {
        ledger.commit_catalog = catalog;
        ledger.commit_envelopes = head_cells;
        ledger.commit_graph = BTreeMap::new();
        ledger.patch_stream_index = BTreeMap::new();
        ledger.cold_catalog = Some(cold);
    });
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
        registry,
    )?;
    history
        .restore_branch_cells(
            &checkpoint.branch_cells,
            &checkpoint.branch_lifecycle,
            &checkpoint.retired_branch_names,
            &roots,
            registry,
        )
        .map_err(corrupt)?;
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
    // G7: only the four fixed-size roots are retained; partitions fault their
    // allocator rows through the working set on first use.
    let record_identity =
        crate::durability::log::native_checkpoint_codec::identity_pages::open_paged_identity(
            directory.manifest.record_identity.clone(),
            directory.identity_member_custody(),
            working_set.cloned(),
        );
    let has_naming_schemes = !registry.persistent_name_schemes().is_empty()
        || regions.schemas.has_persistent_name_schemes();
    Ok(PreparedColdDirectory {
        history,
        symbols: paged_symbols,
        record_identity,
        mirror,
        schemas: regions.schemas,
        has_naming_schemes,
    })
}

pub fn readmit_lineage(
    lineage: &LineageSubsystem,
    checkpoint: &DurableCheckpoint,
    directory: &Arc<crate::durability::data::RelationalCheckpointDirectory>,
    selection: &crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection,
    working_set: Option<Arc<worth_relational_storage::internal_port::storage::residency::working_set::WorkingSetAccountant>>,
    commits: Option<Arc<worth_relational_branch::internal_port::history::commit::cold_catalog::ColdCommitCatalog>>,
    has_naming_schemes: bool,
) -> Result<(), RelationalRecoveryStop> {
    // Event rows resolve their commits through the history's cold catalog,
    // which follows the selected checkpoint and holds no source past close.
    let commits = commits.ok_or_else(|| corrupt("cold lineage has no cold commit catalog"))?;
    let binding = worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::event_residency::LineageCommitBinding(
        Arc::new(move |commit| commits.entry(commit).map(|entry| entry.map(|entry| entry.commit))),
    );
    let may_contain_named_events = has_naming_schemes
        || directory.manifest.naming.branches.iter().any(|root| {
            root.groups.is_some()
                || root.naming.is_some_and(|header| header.unavailable)
                || root.index.is_some_and(|header| header.unavailable)
        });
    lineage.install_cold_pages(
        directory.prepare_lineage_cold_pages(binding, may_contain_named_events)?,
    );
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
    for &(relation_id, lineage_id) in checkpoint.lineage.relation_lineages() {
        lineage.record_relation_lineage(relation_id, lineage_id);
    }
    for envelope in &checkpoint.envelopes {
        for &(relation_id, lineage_id) in envelope.envelope().lineage_relation_lineages() {
            lineage.record_relation_lineage(relation_id, lineage_id);
        }
    }
    // G7 (`P0-A` / `#440`): install per-branch persistent-naming and group
    // state headers lazily behind the selected checkpoint's paged store.
    // Zero lineage node, lineage event, or naming/group pages are read here.
    let naming_source = Arc::new(super::cold_naming::SelectedNamingSource::new(
        directory,
        working_set,
    ));
    lineage.install_cold_naming(naming_source, &directory.manifest.naming);
    selection.register_naming_ack_hook(lineage.naming_ack_hook());
    Ok(())
}
