use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::FoundationalBranchTarget;

use crate::branch::{RelationalBranchCellCheckpoint, RelationalBranchRoot};
use crate::history::data::CommitId;
use crate::history::RelationalCommitCatalog;

use super::HistorySubsystem;

impl HistorySubsystem {
    /// Seal checkpoint envelopes only after the branch images have passed
    /// root readmission. No provisional, unlinked catalog is needed.
    pub(super) fn rebuild_checkpoint_catalog(
        &self,
        checkpoints: &[RelationalBranchCellCheckpoint],
    ) -> Result<(), String> {
        let mut descriptors = BTreeMap::new();
        for checkpoint in checkpoints {
            for target in std::iter::once(checkpoint.observation.target()).chain(
                checkpoint
                    .fork_provenance
                    .as_ref()
                    .map(|provenance| provenance.target()),
            ) {
                let FoundationalBranchTarget::Basis(target) = target else {
                    continue;
                };
                let commit_id = CommitId(target.selected_commit_id());
                match descriptors.get(&commit_id) {
                    Some(existing) if existing != target.roots() => {
                        return Err(format!(
                            "checkpoint carries competing root descriptors for commit `{}`",
                            commit_id.0
                        ));
                    }
                    Some(_) => {}
                    None => {
                        descriptors.insert(commit_id, target.roots().clone());
                    }
                }
            }
        }

        let mut roots: BTreeMap<CommitId, Arc<RelationalBranchRoot>> = BTreeMap::new();
        for cell in self.branch_cells.values() {
            let Some(root) = cell.root() else {
                continue;
            };
            if matches!(cell.observation().target(), FoundationalBranchTarget::Empty) {
                if root.id() != 0 || root.descriptor().is_some() {
                    return Err("empty recovery branch carries a committed root".to_owned());
                }
                continue;
            }
            let commit_id = root
                .commit_id()
                .ok_or_else(|| "live recovery root has no commit identity".to_owned())?;
            roots.entry(commit_id).or_insert(root);
        }

        let mut catalog = RelationalCommitCatalog::default();
        for envelope in self.recorded_commit_envelopes() {
            let commit_id = envelope.commit.commit_id;
            catalog
                .append_checkpoint_recovery(
                    envelope,
                    roots.get(&commit_id),
                    descriptors.get(&commit_id),
                )
                .map_err(|denial| format!("recovered catalog root linkage denied: {denial:?}"))?;
        }
        self.install_commit_catalog(catalog);
        Ok(())
    }
}
