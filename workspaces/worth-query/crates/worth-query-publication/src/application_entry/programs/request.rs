use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};

use super::{
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationProgramAdoptionRequestWithRequirements,
};

pub struct WorthQueryApplicationProgramsRequest<'application, 'principal, 'scope, Schema> {
    pub(super) application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    // Principal-specific adoption authorization enters with Batch 3 custody.
    // Retain the authenticated request binding now so that surface does not drift.
    pub(super) _principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) scope: &'scope WorthQueryRequestScope,
    pub(super) branch: worth_query_execution::facade::product::WorthQueryProductBranch,
}

pub struct WorthQueryApplicationProgramAdoptionRequest<
    'application,
    'principal,
    'scope,
    'target,
    Schema,
> {
    programs: WorthQueryApplicationProgramsRequest<'application, 'principal, 'scope, Schema>,
    target: &'target ApplicationProgramRevision,
}

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationProgramsRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::application_entry) fn new(
        application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &'scope WorthQueryRequestScope,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Self {
        Self {
            application,
            _principal: principal,
            scope,
            branch,
        }
    }

    pub fn compare(
        &self,
        target: &ApplicationProgramRevision,
    ) -> Result<
        WorthQueryProgramAdoptionRequirements,
        WorthQueryApplicationProgramAdoptionPreparationDenial,
    > {
        self.application
            .on_branch(self.branch)
            .select()
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::ProductSelection)?
            .branch_adoption_requirements(target)
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption)
    }

    pub fn adopt<'target>(
        self,
        target: &'target ApplicationProgramRevision,
    ) -> WorthQueryApplicationProgramAdoptionRequest<
        'application,
        'principal,
        'scope,
        'target,
        Schema,
    > {
        WorthQueryApplicationProgramAdoptionRequest {
            programs: self,
            target,
        }
    }
}

impl<'application, 'principal, 'scope, 'target, Schema>
    WorthQueryApplicationProgramAdoptionRequest<'application, 'principal, 'scope, 'target, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn requirements<'requirements>(
        self,
        requirements: &'requirements WorthQueryProgramAdoptionRequirements,
    ) -> WorthQueryApplicationProgramAdoptionRequestWithRequirements<
        'application,
        'principal,
        'scope,
        'target,
        'requirements,
        Schema,
    > {
        WorthQueryApplicationProgramAdoptionRequestWithRequirements {
            programs: self.programs,
            target: self.target,
            requirements,
        }
    }
}
