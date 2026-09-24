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

    /// Selects this exact retained product for read-only application queries.
    pub fn select_on<'a, Schema>(
        &self,
        application: &'a worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<
            'a,
            Schema,
        >,
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        application.select_application_read_observation(&self.retained)
    }

    /// Begins non-authoritative preview custody from this exact retained
    /// occurrence. The returned session carries no publication authority.
    pub fn begin_preview<Schema, Program>(
        &self,
        application: &worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime<
            Schema,
            Program,
        >,
        request: worth_query_execution::facade::application_installation::WorthQueryApplicationPreviewRequest,
    ) -> Result<
        worth_query_execution::facade::application_installation::WorthQueryApplicationPreviewSession,
        worth_query_execution::facade::application_installation::WorthQueryApplicationPreviewReadmissionDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Program: worth_query_declaration::facade::application_program::ApplicationProgramDefinition<
            Schema,
        >,
    {
        application.begin_preview(&self.retained, request)
    }

    pub(super) fn retained_clone(&self) -> Self {
        Self::new(Arc::clone(&self.retained))
    }
}
