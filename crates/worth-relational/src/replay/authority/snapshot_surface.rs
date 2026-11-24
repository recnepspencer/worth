use std::collections::BTreeSet;

use crate::capabilities::HistorySource;
use crate::history::data::{BranchId, CanonicalCommitEnvelope, CommitId};

pub(super) fn snapshot_surface_is_authoritative(
    history: &impl HistorySource,
    envelope: &CanonicalCommitEnvelope,
    closure: &[CommitId],
) -> bool {
    if envelope.branch_context != BranchId("main".to_string()) {
        return false;
    }
    !history.has_committed_version_at_or_before_outside_closure(
        envelope.commit.version_id,
        &closure.iter().copied().collect::<BTreeSet<_>>(),
    )
}
