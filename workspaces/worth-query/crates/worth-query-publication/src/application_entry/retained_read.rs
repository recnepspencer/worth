use std::sync::Arc;

use worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation as RetainedRead;

/// Public read-only handle for one exact World product occurrence.
#[derive(Clone)]
pub struct WorthQueryApplicationReadObservation {
    pub(super) retained: Arc<RetainedRead>,
}

impl WorthQueryApplicationReadObservation {
    pub(super) const fn new(retained: Arc<RetainedRead>) -> Self {
        Self { retained }
    }

    pub fn branch_identity(&self) -> &worth_runtime_world::facade::ProductBranchIdentity {
        self.retained.branch_identity()
    }

    pub fn selected_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.retained.selected_commit()
    }
}
