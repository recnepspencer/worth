//! Admit indexed cells before installing either eager or cold checkpoint routes.
use crate::durability::canonical_commit_artifact::journal_source_read::readmit_journal_commit;
use crate::durability::data::RelationalCheckpointDirectory;
use branch::history::commit::catalog::RelationalCommitCatalog;
use branch::history::commit::index_entry::RelationalCommitIndexEntry;
use branch::history::commit::retained_index::RetainedCanonicalIndex;
use branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop;
pub(super) fn admit(
    execution: &ExecutionRequest<'_, '_>,
    directory: &Arc<RelationalCheckpointDirectory>,
    admitted: impl Fn(&RelationalCommitIndexEntry) -> Option<PositionedCanonicalCommit>,
) -> Result<RelationalCommitCatalog, RelationalRecoveryStop> {
    let mut catalog = RelationalCommitCatalog::default();
    directory.for_each_commit(|entry| -> Result<(), RelationalRecoveryStop> {
        let actual =
            RetainedCanonicalIndex::copy_original(entry, execution, readmit_journal_commit)
                .map_err(RelationalRecoveryStop::Request)?;
        let artifact = branch::history::commit::artifact::RelationalCommitArtifact::from_index(
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
        Ok(())
    })?;
    Ok(catalog)
}
