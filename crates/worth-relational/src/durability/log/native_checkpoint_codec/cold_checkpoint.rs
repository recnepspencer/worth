//! Readmit only the directory and the envelopes of its distinct heads.
use std::sync::Arc;

use super::{manifest_capture::corrupt, manifest_validation::ManifestDirectoryFacts};
use crate::durability::data::*;
use crate::durability::medium::RelationalDurableColdSource;

pub(super) fn readmit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    manifest: RelationalCheckpointManifest,
    source: Arc<dyn RelationalDurableColdSource>,
    facts: ManifestDirectoryFacts,
) -> Result<DurableCheckpoint, RelationalRecoveryStop> {
    let directory = Arc::new(
        PreparedCheckpointDirectory {
            manifest: manifest.clone(),
            source,
            history_members: facts.history_members,
        }
        .readmit(execution)?,
    );
    let heads = facts.heads;
    let mut envelopes = Vec::new();
    let mut symbols = crate::symbols::data::StringInterner::default();
    for entry in manifest
        .commits
        .iter()
        .filter(|entry| heads.contains(&entry.commit.commit_id))
    {
        let commit = crate::history::RelationalCommitEnvelopeResidency::readmit_selected(
            entry,
            &directory,
            execution,
            1 << 30,
        )?;
        symbols
            .import_canonical_snapshot(&commit.symbol_texts)
            .map_err(corrupt)?;
        envelopes.push(commit);
    }
    let manifest = manifest.ordinary_owned_draft();
    let (runtime_name, branch_cells, branch_lifecycle, retired_branch_names) =
        manifest.branch_data.ordinary_owned_parts();
    let checkpoint = DurableCheckpoint {
        recorded_components: manifest.roots.iter().map(|root| (root.commit_id, root.recorded_components.clone())).collect(),
        retained_empty_components: manifest.retained_empty_components.clone(),
        root_kind_images: Default::default(), mirror_kind_images: Default::default(),
        directory: Some(directory), canonical_directory:None,
        coverage: manifest.coverage,
        commit_descriptors: manifest.commits.iter().map(|entry| (entry.commit.commit_id, entry.root.clone())).collect(),
        mirror_schema_digest: manifest.mirror_schema,
        branch_cells, branch_lifecycle,
        retired_branch_names,
        branch_roots: Vec::new(), branch_root_schema_images: manifest.schemas.ordinary_owned_copy(),
        record_identity: manifest.record_identity,
        record_generation_high_water: Vec::new(), reusable_record_slots: Vec::new(), record_slot_frontiers: Vec::new(),
        envelopes, partition_images: Vec::new(), aspect_contracts: manifest.aspect_contracts,
        lineage: crate::lineage::data::LineageCheckpointArtifact::new(manifest.lineage_basis.ordinary_owned_copy(), Vec::new()),
        index_generation_frontier: manifest.index_generation_frontier,
        index_definition_frontier: manifest.index_definition_frontier,
        index_definitions: manifest.index_definitions.ordinary_owned_copy(), derived_index_artifacts: Default::default(),
        derived_index_checkpoint: None,
        derived_index_checkpoint_format: crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::FORMAT_VERSION,
        symbol_table: symbols.snapshot(), symbol_identity_frontier: manifest.symbol_identity_frontier,
        runtime_name,
    };
    Ok(checkpoint)
}
