//! Readmit only the directory and the envelopes of its distinct heads.
use std::sync::Arc;

use super::{manifest_capture::corrupt, manifest_validation};
use crate::durability::canonical_commit_artifact::{
    RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactRefusal,
};
use crate::durability::data::*;
use crate::durability::medium::RelationalDurableColdSource;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

pub(crate) fn readmit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    manifest: RelationalCheckpointManifest,
    source: Arc<dyn RelationalDurableColdSource>,
) -> Result<DurableCheckpoint, RelationalRecoveryStop> {
    let heads = manifest_validation::validate(&manifest)?;
    let mut envelopes = Vec::new();
    let mut symbols = crate::symbols::data::StringInterner::default();
    for entry in manifest
        .commits
        .iter()
        .filter(|entry| heads.contains(&entry.commit.commit_id))
    {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let bytes = source
            .read_commit(entry.position, 1 << 30)
            .map_err(|cause| RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Envelope(entry.position),
                cause,
            })?;
        let artifact = RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes)
            .map_err(|error| corrupt(format!("head envelope frame: {error:?}")))?;
        if artifact.position() != entry.position
            || artifact.commit_id() != entry.commit.commit_id
            || artifact.payload_digest().as_bytes() != &entry.payload_digest
        {
            return Err(corrupt("head envelope disagrees with its commit index").into());
        }
        let commit = RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), execution)
            .map_err(artifact_stop)?
            .into_positioned();
        if commit.commit != entry.commit || commit.schema_version != entry.schema_version {
            return Err(corrupt("head envelope identity disagrees with its commit index").into());
        }
        symbols
            .import_canonical_snapshot(&commit.symbol_texts)
            .map_err(corrupt)?;
        envelopes.push(commit);
    }
    let checkpoint = DurableCheckpoint {
        directory: Some(Arc::new(RelationalCheckpointDirectory { manifest: manifest.clone(), source })),
        coverage: manifest.coverage,
        commit_descriptors: manifest.commits.iter().map(|entry| (entry.commit.commit_id, entry.root.clone())).collect(),
        mirror_schema_digest: manifest.mirror_schema,
        branch_cells: manifest.branch_cells, branch_lifecycle: manifest.branch_lifecycle,
        retired_branch_names: manifest.retired_branch_names,
        branch_roots: Vec::new(), branch_root_schema_images: manifest.schemas,
        record_identity: manifest.record_identity,
        record_generation_high_water: Vec::new(), reusable_record_slots: Vec::new(), record_slot_frontiers: Vec::new(),
        envelopes, partition_images: Vec::new(), aspect_contracts: manifest.aspect_contracts,
        lineage: crate::lineage::data::LineageCheckpointArtifact::new(manifest.lineage_basis, Vec::new()),
        index_definitions: manifest.index_definitions, derived_index_artifacts: Default::default(),
        derived_index_checkpoint: None,
        derived_index_checkpoint_format: crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::FORMAT_VERSION,
        symbol_table: symbols.snapshot(), symbol_identity_frontier: manifest.symbol_identity_frontier,
        runtime_name: manifest.runtime_name,
    };
    Ok(checkpoint)
}

pub(super) fn artifact_stop(
    error: RelationalCanonicalCommitArtifactRefusal,
) -> RelationalRecoveryStop {
    match error {
        RelationalCanonicalCommitArtifactRefusal::Request(cause) => {
            RelationalRecoveryStop::Request(cause)
        }
        RelationalCanonicalCommitArtifactRefusal::FieldText(cause) => {
            crate::symbols::data::field_text_recovery_stop(cause)
        }
        error => corrupt(format!("checkpoint head readmission: {error:?}")).into(),
    }
}
