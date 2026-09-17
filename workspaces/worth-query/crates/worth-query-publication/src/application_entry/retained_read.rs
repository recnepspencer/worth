use std::sync::Arc;

use worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation as RetainedRead;

/// Public read-only handle for one exact World product occurrence.
pub struct WorthQueryApplicationReadObservation {
    pub(super) retained: Arc<RetainedRead>,
}

impl WorthQueryApplicationReadObservation {
    pub(super) const fn new(retained: Arc<RetainedRead>) -> Self {
        Self { retained }
    }

    pub fn retain_on_branch<Schema>(
        application: &worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<
        Self,
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        let selected = application.on_branch(branch).select()?;
        Ok(Self::new(selected.retain_application_read()))
    }

    pub fn branch_identity(&self) -> &worth_runtime_world::facade::ProductBranchIdentity {
        self.retained.branch_identity()
    }

    pub fn selected_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.retained.selected_commit()
    }

    /// Retains another read-only lease for this exact product occurrence.
    pub fn retain(&self) -> Self {
        Self::new(Arc::clone(&self.retained))
    }

    pub(super) fn retained_clone(&self) -> Self {
        Self::new(Arc::clone(&self.retained))
    }
}
