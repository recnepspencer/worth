use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryApplicationRequest;
use crate::application_entry::programs::WorthQueryApplicationBranchSetProgramsRequest;

/// Request context bound to one exact owner-issued branch coverage and one
/// explicit non-atomic publication order.
pub struct WorthQueryApplicationBranchSetRequest<'application, 'principal, 'scope, Schema> {
    pub(in crate::application_entry) application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(in crate::application_entry) principal:
        &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(in crate::application_entry) scope: &'scope WorthQueryRequestScope,
    pub(in crate::application_entry) coverage:
        worth_query_execution::facade::primary_graph::WorthQueryOrderedProgramAdoptionCoverage,
}

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn on_branches(
        self,
        coverage: worth_query_execution::facade::primary_graph::WorthQueryProgramAdoptionCoverage,
        ordered_targets: &[worth_query_execution::facade::product::WorthQueryProductBranch],
    ) -> Result<
        WorthQueryApplicationBranchSetRequest<'application, 'principal, 'scope, Schema>,
        worth_query_execution::facade::primary_graph::WorthQueryProgramAdoptionCoverageDenial,
    > {
        let coverage = self
            .application
            .branches()
            .order_program_adoption_coverage(coverage, ordered_targets)?;
        Ok(WorthQueryApplicationBranchSetRequest {
            application: self.application,
            principal: self.principal,
            scope: self.scope,
            coverage,
        })
    }

    /// Continues exact unpublished custody without separating it from the
    /// branch-set's performed prefix or untouched suffix.
    pub fn recover_branch_set_adoption(
        self,
        recovery: worth_query_execution::facade::primary_graph::WorthQueryBranchSetAdoptionRecovery,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryBranchSetAdoptionRecoveryOutcome,
        worth_query_execution::facade::primary_graph::WorthQueryBranchSetAdoptionRecoveryFailure,
    > {
        self.application
            .recover_branch_set_adoption(recovery, self.scope)
    }

    /// Releases exact unpublished custody and cancels only the untouched
    /// suffix while retaining the performed prefix as terminal evidence.
    pub fn release_branch_set_adoption_recovery(
        self,
        recovery: worth_query_execution::facade::primary_graph::WorthQueryBranchSetAdoptionRecovery,
        minimum_age_ticks: u64,
    ) -> Result<
        (
            worth_query_execution::facade::primary_graph::WorthQueryBranchSetAdoptionCancellation,
            worth_query_execution::facade::product::WorthQueryProductBranchOwnerCleanupReceipt,
        ),
        worth_query_execution::facade::primary_graph::WorthQueryBranchSetAdoptionRecoveryReleaseFailure,
    >{
        self.application
            .release_branch_set_adoption_recovery(recovery, minimum_age_ticks)
    }
}

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationBranchSetRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn programs(
        self,
    ) -> WorthQueryApplicationBranchSetProgramsRequest<'application, 'principal, 'scope, Schema>
    {
        WorthQueryApplicationBranchSetProgramsRequest::new(
            self.application,
            self.principal,
            self.scope,
            self.coverage,
        )
    }
}
