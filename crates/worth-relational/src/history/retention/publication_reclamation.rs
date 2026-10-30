//! Ordinary generation maintenance receives the original sealed publication.

use std::collections::BTreeSet;

use crate::history::data::CommitId;
use crate::identity::data::VersionId;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::CommitResult;
use worth_foundational::FoundationalBranchTarget;

impl RelationalRuntime {
    /// Retire unselected derived index generations after this owner's settled
    /// publication. Work follows its previous bases and affected index scopes,
    /// plus at most the configured retention batch of older generations.
    ///
    /// Release temporary before-basis and settlement holds first. Intentional
    /// readers and retained receipts keep their exact generations selectable.
    /// `None` defers maintenance while publication is not quiescent. Payloads
    /// are destroyed after releasing catalog and canonical publication locks.
    pub fn run_index_generation_reclamation_after_publication(
        &self,
        committed: &CommitResult,
    ) -> Result<Option<usize>, crate::mvcc::RelationalPublicationDenial> {
        let actual_runtime_instance_id = committed.snapshot.runtime_instance_id();
        if actual_runtime_instance_id != self.runtime_instance_id() {
            return Err(crate::mvcc::RelationalPublicationDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime_instance_id(),
                actual_runtime_instance_id,
            });
        }
        let retired = self
            .history
            .canonical_checkpoint_gate()
            .with_quiescent_index_reclamation(|| {
                let publication = committed.envelope();
                let mut previous_commits: BTreeSet<_> =
                    publication.commit.parents.iter().copied().collect();
                let mut previous_versions = BTreeSet::<VersionId>::new();
                if let Some(checkpoint) = &publication.branch_cell_checkpoint
                    && let FoundationalBranchTarget::Basis(previous) =
                        checkpoint.observation.target()
                {
                    previous_commits.insert(CommitId(previous.selected_commit_id()));
                    previous_versions.insert(VersionId(previous.version_id()));
                }
                for commit in previous_commits {
                    if let Some(artifact) = self.history.commit_artifact(commit) {
                        previous_versions.insert(artifact.version_id());
                    }
                }
                self.indexes.retire_after_publication(
                    &self.history,
                    publication,
                    &previous_versions,
                    self.config.storage.retention.reclaim_batch_size,
                )
            });
        Ok(retired.map(|generations| {
            let count = generations.len();
            drop(generations);
            count
        }))
    }
}
