use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;

pub struct WorthQueryApplicationBranchSetProgramsRequest<
    'application,
    'principal,
    'scope,
    Schema,
> {
    application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    _principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &'scope WorthQueryRequestScope,
    coverage: worth_query_execution::facade::primary_graph::WorthQueryOrderedProgramAdoptionCoverage,
}

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationBranchSetProgramsRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::application_entry) fn new(
        application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &'scope WorthQueryRequestScope,
        coverage: worth_query_execution::facade::primary_graph::WorthQueryOrderedProgramAdoptionCoverage,
    ) -> Self {
        Self {
            application,
            _principal: principal,
            scope,
            coverage,
        }
    }

    pub fn adopt<'target>(
        self,
        target: &'target ApplicationProgramRevision,
    ) -> WorthQueryApplicationBranchSetProgramAdoptionRequest<
        'application,
        'principal,
        'scope,
        'target,
        Schema,
    > {
        WorthQueryApplicationBranchSetProgramAdoptionRequest {
            request: self,
            target,
        }
    }
}

pub struct WorthQueryApplicationBranchSetProgramAdoptionRequest<
    'application,
    'principal,
    'scope,
    'target,
    Schema,
> {
    request:
        WorthQueryApplicationBranchSetProgramsRequest<'application, 'principal, 'scope, Schema>,
    target: &'target ApplicationProgramRevision,
}

impl<'application, 'principal, 'scope, 'target, Schema>
    WorthQueryApplicationBranchSetProgramAdoptionRequest<
        'application,
        'principal,
        'scope,
        'target,
        Schema,
    >
where
    Schema: ApplicationSchema,
{
    pub fn prepare(
        self,
        maximum_selection_work_per_branch: usize,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryPreparedBranchSetAdoption,
        worth_query_execution::facade::primary_graph::WorthQueryBranchSetAdoptionPreparationDenial,
    > {
        self.request.application.prepare_branch_set_adoption(
            self.request.coverage,
            self.target,
            maximum_selection_work_per_branch,
            self.request.scope,
        )
    }
}
