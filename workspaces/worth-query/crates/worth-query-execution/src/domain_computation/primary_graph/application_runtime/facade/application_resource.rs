//! The runtime's consumer methods over `domain_computation::primary_graph::application_resource`; each only calls its subsystem.
use crate::domain_computation::authorization::application_disclosure::prepare_pending_disclosure;
use crate::domain_computation::authorization::application_disclosure::WorthQueryPendingApplicationQueryGovernance;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
use worth_query_declaration::facade::application_resource::ApplicationResourceBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

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
