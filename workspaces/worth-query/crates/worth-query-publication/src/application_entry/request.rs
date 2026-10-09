use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
use worth_query_declaration::facade::application_query::ApplicationQueryIntent;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryApplicationMutationRequest, WorthQueryApplicationOutputDemandRequest,
    WorthQueryApplicationProgramsRequest, WorthQueryApplicationQueryRequest,
};
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

mod branch_set;
mod output_currentness;
mod program_outputs;
pub use branch_set::WorthQueryApplicationBranchSetRequest;
pub use output_currentness::{
    WorthQueryOutputCurrentnessDenial, WorthQueryProgramOutputCurrentnessDenial,
};

/// Borrowed ordinary-request context. Construction selects no World state and
/// resolves no application principal.
pub struct WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema> {
    pub(super) application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) scope: &'scope WorthQueryRequestScope,
    pub(super) branch: worth_query_execution::facade::product::WorthQueryProductBranch,
}

/// A request pinned to one retained read observation, from `at`, `at_commit` or
/// `at_approved_elevation`. Its queries and demands read that observation.
pub struct WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema> {
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &'scope WorthQueryRequestScope,
    branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    observation: std::sync::Arc<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
    >,
}

/// Why a retained request could not be pinned. `CommitUnavailable` means the commit is not
/// in the branch history within the work bound.
#[derive(Debug)]
pub enum WorthQueryApplicationHistorySelectionDenial {
    ProductSelection(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    CommitUnavailable,
}

/// Starts an application request. `request` binds a principal and scope to the runtime's
/// current world.
pub trait WorthQueryApplicationRequestExt<Schema>
where
    Schema: ApplicationSchema,
{
    fn request<'application, 'principal, 'scope>(
        &'application self,
        principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &'scope WorthQueryRequestScope,
    ) -> WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>;
}

impl<Schema> WorthQueryApplicationRequestExt<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    fn request<'application, 'principal, 'scope>(
        &'application self,
        principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &'scope WorthQueryRequestScope,
    ) -> WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema> {
        WorthQueryApplicationRequest {
            application: self,
            principal,
            scope,
            branch: self.current_world(),
        }
    }
}

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema,
{
    /// Targets all operations built from this request at one exact World product occurrence.
    pub fn on_branch(
        mut self,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Self {
        self.branch = branch;
        self
    }

    pub fn query<Intent>(
        &self,
        intent: Intent,
    ) -> WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent>
    where
        Intent: ApplicationQueryIntent<Schema>,
    {
        WorthQueryApplicationQueryRequest::new(
            self.application,
            self.principal,
            self.scope,
            self.branch,
            intent,
        )
    }

    pub fn mutate<Intent>(
        &self,
        intent: Intent,
    ) -> WorthQueryApplicationMutationRequest<'application, 'principal, 'scope, Schema, Intent>
    where
        Intent: ApplicationMutationIntent<Schema>,
    {
        WorthQueryApplicationMutationRequest::new(
            self.application,
            self.principal,
            self.scope,
            self.branch,
            intent,
        )
    }

    pub fn demand<Demand>(
        &self,
        demand: Demand,
    ) -> WorthQueryApplicationOutputDemandRequest<'application, 'principal, 'scope, Schema, Demand>
    where
        Demand: worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand<Schema>,
    {
        WorthQueryApplicationOutputDemandRequest::new(
            self.application,
            self.principal,
            self.scope,
            self.branch,
            demand,
        )
    }

    /// Enters branch-local application-program inspection and adoption.
    pub fn programs(
        &self,
    ) -> WorthQueryApplicationProgramsRequest<'application, 'principal, 'scope, Schema> {
        WorthQueryApplicationProgramsRequest::new(
            self.application,
            self.principal,
            self.scope,
            self.branch,
        )
    }

    pub fn at(
        &self,
        observation: &super::WorthQueryApplicationReadObservation,
    ) -> WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema> {
        WorthQueryApplicationRetainedRequest {
            application: self.application,
            principal: self.principal,
            scope: self.scope,
            branch: self.branch,
            observation: std::sync::Arc::clone(&observation.retained),
        }
    }

    /// Selects the exact committed product occurrence identified by a receipt.
    pub fn at_commit(
        &self,
        commit: &worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        maximum_history_work: std::num::NonZeroUsize,
    ) -> Result<
        WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema>,
        WorthQueryApplicationHistorySelectionDenial,
    > {
        self.at_selected_commit(
            commit.committed_product_publication().composite_commit(),
            maximum_history_work,
        )
    }

    /// Selects the exact product occurrence that issued an approved elevation.
    pub fn at_approved_elevation(
        &self,
        approved: &worth_query_execution::facade::primary_graph::WorthQueryApprovedElevation,
        maximum_history_work: std::num::NonZeroUsize,
    ) -> Result<
        WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema>,
        WorthQueryApplicationHistorySelectionDenial,
    > {
        self.at_selected_commit(
            approved.approval_product_publication().composite_commit(),
            maximum_history_work,
        )
    }

    fn at_selected_commit(
        &self,
        selected_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
        maximum_history_work: std::num::NonZeroUsize,
    ) -> Result<
        WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema>,
        WorthQueryApplicationHistorySelectionDenial,
    > {
        let history = self
            .application
            .branches()
            .history(self.branch, maximum_history_work)
            .map_err(WorthQueryApplicationHistorySelectionDenial::ProductSelection)?;
        let entry = history
            .entries()
            .find(|entry| entry.selected_commit() == selected_commit)
            .ok_or(WorthQueryApplicationHistorySelectionDenial::CommitUnavailable)?;
        let selected = history
            .select(&entry)
            .map_err(WorthQueryApplicationHistorySelectionDenial::ProductSelection)?;
        Ok(WorthQueryApplicationRetainedRequest {
            application: self.application,
            principal: self.principal,
            scope: self.scope,
            branch: self.branch,
            observation: selected.retain_application_read(),
        })
    }

    pub fn retain_read(
        &self,
    ) -> Result<
        super::WorthQueryApplicationReadObservation,
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    > {
        let selected = self.application.on_branch(self.branch).select()?;
        Ok(super::WorthQueryApplicationReadObservation::new(
            selected.retain_application_read(),
        ))
    }
}

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema,
{
    /// Retains another read-only lease for this request's exact observation.
    /// Unlike an ordinary request's `retain_read`, this does not select branch
    /// head or traverse history. The lease survives this request and later
    /// branch publications; it grants no mutation authority.
    pub fn retain_read(&self) -> super::WorthQueryApplicationReadObservation {
        super::WorthQueryApplicationReadObservation::new(std::sync::Arc::clone(&self.observation))
    }

    /// Reads independently scoped intents at this same observation, publishing
    /// only a complete ordered batch. Duplicate scopes are independently billed.
    pub fn query_batch<Intent>(
        &self,
        intents: Vec<Intent>,
    ) -> super::WorthQueryApplicationQueryBatchRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
    >
    where
        Intent: ApplicationQueryIntent<Schema>,
    {
        super::WorthQueryApplicationQueryBatchRequest::new(
            self.application,
            self.principal,
            self.scope,
            self.branch,
            std::sync::Arc::clone(&self.observation),
            intents,
        )
    }

    pub fn query<Intent>(
        &self,
        intent: Intent,
    ) -> WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent>
    where
        Intent: ApplicationQueryIntent<Schema>,
    {
        WorthQueryApplicationQueryRequest::new_at(
            self.application,
            self.principal,
            self.scope,
            self.branch,
            std::sync::Arc::clone(&self.observation),
            intent,
        )
    }

    pub fn demand<Demand>(
        &self,
        demand: Demand,
    ) -> WorthQueryApplicationOutputDemandRequest<'application, 'principal, 'scope, Schema, Demand>
    where
        Demand: worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand<Schema>,
    {
        WorthQueryApplicationOutputDemandRequest::new_at(
            self.application,
            self.principal,
            self.scope,
            self.branch,
            std::sync::Arc::clone(&self.observation),
            demand,
        )
    }
}
