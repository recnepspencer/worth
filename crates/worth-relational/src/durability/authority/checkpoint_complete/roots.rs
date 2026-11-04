//! Actual owner-held current/external roots, never a historical mutation fold.
use super::Denial;
use crate::branch::{RelationalBranchBasisDescriptor, RelationalBranchRoot};
use crate::history::data::CommitId;
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

pub(super) struct CapturedRoots {
    rows: ExecutionOrderedMap<CommitId, Arc<RelationalBranchRoot>>,
}
impl CapturedRoots {
    pub(super) fn capture<'a>(
        observations: impl Iterator<
            Item = (
                &'a RelationalBranchBasisDescriptor,
                &'a Arc<RelationalBranchRoot>,
            ),
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut rows = ExecutionOrderedMap::default();
        for (_, root) in observations {
            crate::execution::check_request(request).map_err(Denial::Request)?;
            let Some(commit) = root.commit_id() else {
                continue;
            };
            if let Some(prior) = rows.get(&commit) {
                let prior: &Arc<RelationalBranchRoot> = prior;
                if prior.descriptor() != root.descriptor()
                    || prior.axes() != root.axes()
                    || prior.commit_receipt() != root.commit_receipt()
                {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
            } else {
                rows.insert_cloned_native_for_request(&commit, root, *request)
                    .map_err(|cause| {
                        crate::durability::checkpoint_history::capture::capture_stop(
                            crate::durability::checkpoint_history::capture::map_stop(cause),
                        )
                    })?;
            }
        }
        Ok(Self { rows })
    }
    pub(super) fn get(&self, commit: CommitId) -> Option<&Arc<RelationalBranchRoot>> {
        self.rows.get(&commit)
    }
}
