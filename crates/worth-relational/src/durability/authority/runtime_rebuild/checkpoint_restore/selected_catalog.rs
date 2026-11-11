//! Admit indexed cells before installing either eager or cold checkpoint routes.
use crate::durability::data::{RelationalCheckpointDirectory, RelationalRecoveryStop};
use crate::history::data::PositionedCanonicalCommit;
use crate::history::{RelationalCommitCatalog, RelationalCommitIndexEntry, RetainedCanonicalIndex};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
pub(super) fn admit(
    execution: &ExecutionRequest<'_, '_>,
    directory: &Arc<RelationalCheckpointDirectory>,
    admitted: impl Fn(&RelationalCommitIndexEntry) -> Option<PositionedCanonicalCommit>,
) -> Result<RelationalCommitCatalog, RelationalRecoveryStop> {
    let mut catalog = RelationalCommitCatalog::default();
    for entry in &directory.manifest.commits {
        let actual = RetainedCanonicalIndex::copy_original(entry, execution)
            .map_err(RelationalRecoveryStop::Request)?;
        let artifact = crate::history::RelationalCommitArtifact::from_index(
            actual.clone(),
            directory.source.clone(),
        )
        .map_err(|cause| {
            crate::durability::checkpoints::region_directory::corrupt(format!(
                "indexed commit admission: {cause:?}"
            ))
        })?;
        if let Some(admitted) = admitted(entry) {
            artifact
                .envelope_residency()
                .bind_recovered_positioned(execution, &admitted)?;
        }
        let artifact = artifact
            .prepare_catalog_owner(execution)
            .map_err(RelationalRecoveryStop::Request)?;
        catalog
            .append_preencoded_recovery(artifact)
            .map_err(|cause| {
                crate::durability::checkpoints::region_directory::corrupt(format!(
                    "indexed commit insertion: {cause:?}"
                ))
            })?;
        let cell = catalog
            .get(entry.commit.commit_id)
            .ok_or_else(|| {
                crate::durability::checkpoints::region_directory::corrupt(
                    "admitted indexed commit unavailable",
                )
            })?
            .envelope_residency();
        // The new cell and this handoff share the SAME already admitted entry owner.
        let _displaced = cell
            .install_selected(actual, directory.clone())
            .map_err(|cause| {
                crate::durability::checkpoints::region_directory::corrupt(format!(
                    "selected canonical placement: {cause:?}"
                ))
            })?;
    }
    Ok(catalog)
}
