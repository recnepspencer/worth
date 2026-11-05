//! The runtime's consumer methods over `domain_computation::primary_graph::application_resource`; each only calls its subsystem.
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn application_resources<'a, 'p, 'r>(
        &'a self,
        external: &'p WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &'r WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> crate::domain_computation::primary_graph::application_resource::WorthQueryApplicationResourcesRequest<'a, 'p, 'r, Schema>{
        <Self as crate::domain_computation::primary_graph::application_resource::RequestOps<
            Schema,
        >>::application_resources(self, external, request, branch)
    }
}
