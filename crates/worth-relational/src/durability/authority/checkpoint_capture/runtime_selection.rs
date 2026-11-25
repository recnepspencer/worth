//! Capture the actual runtime roots under one performed publication selection.
use super::CapturedCheckpointBasis;
use crate::capabilities::RuntimeIdentitySource;
use crate::durability::data::DurabilityError;
use crate::runtime::RelationalRuntime;
use worth_relational_contracts::internal_port::symbols::data::string_interner::StringInternerOwnerPort as _;

/// Select one immutable recovery basis while publication handoff is
/// excluded. Expensive checkpoint reconstruction, encoding, and I/O occur
/// only after the caller drops the admission guard.
pub(in super::super) fn capture(
    runtime: &RelationalRuntime,
    routes: &crate::runtime::RelationalCanonicalPublicationRoutes,
    selection: crate::runtime::PerformedCheckpointSelection,
) -> Result<CapturedCheckpointBasis, DurabilityError> {
    let record_identity = runtime.record_identity.checkpoint_identity();
    // Checkpointing is a cold lane: retire expired reader roots and their
    // derived generations before selecting bounded payloads.
    runtime.run_index_generation_reclamation_pass();
    let envelopes = selection.positioned_snapshot();
    let selected_directory = runtime.durability.checkpoint_selection().snapshot();
    let artifacts = runtime.history.commit_artifacts();
    let lineage_basis = super::super::checkpoint_lineage_basis::capture(
        &artifacts,
        selected_directory
            .as_ref()
            .map(|directory| &directory.manifest),
    )?;
    let journal_index = artifacts
        .iter()
        .filter_map(|artifact| {
            artifact
                .envelope_residency()
                .journal_index()
                .map(|entry| entry.as_ref().clone())
        })
        .collect();
    let covered_transitions = selection.covered_transitions();
    let components = super::super::checkpoint_components::capture(&runtime.history)?;
    let branch_roots = components.roots;
    let branch_cells = runtime.history().branch_cells_snapshot();
    let (retained, mirror_schema_authority) = super::capture_root_links(
        &artifacts,
        &branch_roots,
        &branch_cells,
        runtime.main_branch_identity().branch_id(),
    )?;
    let (lineage_event_leases, lineage_event_pages_resident) = runtime
        .lineage
        .capture_resident_event_pages()
        .map_err(DurabilityError::from)?;
    let published_rows = runtime.lineage.published_rows();
    let lineage_events = super::captured_lineage(published_rows, &lineage_event_leases)?;
    let (index_state, index_original) = runtime.indexes.snapshot_with_original();
    let derived_index_checkpoint = super::capture_index_checkpoint(&index_state, &retained)?;
    let captured = CapturedCheckpointBasis {
        recorded_components: components.canonical,
        retained_empty_components: components.empty,
        empty_component_schemas: components.empty_schemas,
        component_holds: components.held,
        envelope_cells: artifacts
            .iter()
            .map(|artifact| artifact.envelope_residency())
            .collect(),
        journal_index,
        lineage_basis,
        latest_commit: selection.latest_receipt().or_else(|| {
            selected_directory.as_ref().and_then(|directory| {
                directory
                    .manifest
                    .commits
                    .last()
                    .map(|entry| entry.commit.clone())
            })
        }),
        selected_directory,
        branch_cells,
        branch_lifecycle: runtime.history.branch_lifecycle_snapshot(),
        retired_branch_names: runtime.history.retired_branch_names(),
        covered_transitions,
        branch_roots,
        record_identity,
        commit_descriptors: artifacts
            .iter()
            .map(|artifact| (artifact.identity().commit_id(), artifact.roots().clone()))
            .collect(),
        envelopes,
        partitions: runtime.acquire_partition_edition(),
        mirror_schema_authority,
        aspect_contracts: runtime
            .schema_contract_runtime
            .aspect_contract_plans
            .as_ref()
            .clone(),
        lineage_nodes: runtime.lineage_access().resident_nodes_snapshot(),
        relation_lineages: runtime.lineage.relation_lineages_snapshot(),
        lineage_event_pages_resident,
        lineage_event_leases,
        guarded_member_sources: runtime
            .lineage
            .event_member_sources()
            .into_iter()
            .chain(runtime.lineage.node_export_sources())
            .chain(runtime.services.symbols.checkpoint_export_sources())
            .chain(
                index_original
                    .clone()
                    .into_iter()
                    .flat_map(|image| image.checkpoint_export_sources()),
            )
            .collect(),
        lineage_events,
        index_definitions: index_state.definition_snapshot(),
        carried_index_image: index_original
            .as_ref()
            .map(|image| image.directory().manifest.derived_indexes),
        index_state,
        index_original,
        retained_indexes: retained,
        derived_index_checkpoint,
        symbol_table: runtime.services.symbols.snapshot(),
        symbol_identity_frontier: runtime
            .services
            .symbols
            .with_read(|symbols| symbols.identity_frontier()),
        symbol_pages_resident: runtime.services.symbols.checkpoint_contents_resident(),
        runtime_name: runtime.runtime_name().to_string(),
    };
    routes
        .validate_checkpoint_selection(&selection)
        .map_err(super::super::checkpointing::checkpoint_admission_error)?;
    Ok(captured)
}
