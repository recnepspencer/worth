use std::collections::BTreeSet;

use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::VersionId;
use crate::runtime::RelationalRuntime;

pub(crate) trait HistorySource: super::CommitEnvelopeSource {
    fn branch_head_ref(&self, branch_id: &BranchId) -> Option<RelationalCommitReceipt>;
    /// The catalog's sealed envelopes, carried by shared ownership so the
    /// caller never holds the history ledger while scanning them.
    fn authoritative_commit_envelopes(&self) -> Vec<std::sync::Arc<CanonicalCommitEnvelope>>;

    fn has_committed_version_at_or_before_outside_closure(
        &self,
        version_id: VersionId,
        closure: &BTreeSet<CommitId>,
    ) -> bool {
        self.authoritative_commit_envelopes()
            .into_iter()
            .any(|candidate| {
                candidate.commit.version_id <= version_id
                    && !closure.contains(&candidate.commit.commit_id)
            })
    }
}

impl HistorySource for RelationalRuntime {
    fn branch_head_ref(&self, branch_id: &BranchId) -> Option<RelationalCommitReceipt> {
        let cell = self.history.branch_cell(branch_id)?;
        let commit_id = match cell.observation().target() {
            worth_foundational::FoundationalBranchTarget::Empty => return None,
            worth_foundational::FoundationalBranchTarget::Basis(target) => {
                CommitId(target.selected_commit_id())
            }
        };
        self.history
            .commit_artifact(commit_id)
            .map(|artifact| artifact.receipt())
    }

    fn authoritative_commit_envelopes(&self) -> Vec<std::sync::Arc<CanonicalCommitEnvelope>> {
        self.history.catalog_envelopes()
    }
}
