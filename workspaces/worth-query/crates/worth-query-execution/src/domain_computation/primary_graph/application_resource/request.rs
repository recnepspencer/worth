//! Public Resource actions delegate to the installed runtime's original request.
mod existing;
mod inspection;
mod opening;
mod owned_effect;
mod progression;
mod start;
use super::{installation::InstalledResourceBinding, stop::ResourceCause};
use crate::domain_computation::{
    authorization::{
        application_disclosure::{
            prepare_pending_disclosure, WorthQueryPendingApplicationQueryGovernance,
        },
        WorthQueryAdmittedApplicationCapabilityAccess,
    },
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
};
pub use inspection::{WorthQueryResourceCleanupDetail, WorthQueryResourceState};
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
    application_resource::ApplicationResourceBinding,
    application_schema::ApplicationSchema,
};
use worth_relational::facade::identity::EntityId;

/// Fresh authenticated actions on the runtime's installed Resource family.
pub struct WorthQueryApplicationResourcesRequest<'application, 'principal, 'request, Schema> {
    runtime: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &'request WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
}

type AccessScope<Schema, Binding> = <<<Binding as ApplicationResourceBinding<Schema>>::Access as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Enters the actual installed Resource action family. The selected branch
    /// is used only by Start; retained actions freshly select their original branch.
    pub fn application_resources<'a, 'p, 'r>(
        &'a self,
        external: &'p WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &'r WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> WorthQueryApplicationResourcesRequest<'a, 'p, 'r, Schema> {
        WorthQueryApplicationResourcesRequest {
            runtime: self,
            external,
            request,
            branch,
        }
    }
}

impl<Schema: ApplicationSchema> WorthQueryApplicationResourcesRequest<'_, '_, '_, Schema> {
    fn installed<Binding: ApplicationResourceBinding<Schema>>(
        &self,
    ) -> Result<Arc<InstalledResourceBinding<Schema, Binding>>, ResourceCause> {
        self.runtime
            .installed_resources
            .binding::<Binding>()
            .ok_or(ResourceCause::Unavailable)
    }

    fn pending<Binding, Capability, Operation, Input>(
        &self,
        installed: &InstalledResourceBinding<Schema, Binding>,
        principal: EntityId,
        scope: EntityId,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<Option<WorthQueryPendingApplicationQueryGovernance>, ResourceCause>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        if let Some(cause) = capability.request_scope().interruption() {
            return Err(ResourceCause::Control(cause));
        }
        prepare_pending_disclosure(
            self.runtime,
            installed
                .installed
                .access_binding()
                .query()
                .binding_identity(),
            principal,
            scope,
            capability,
            || {
                self.request
                    .interruption()
                    .map_or(Ok(()), |cause| Err(ResourceCause::Control(cause)))
            },
            || {
                ResourceCause::Admission(
                    super::admission::ResourceAdmissionDenial::ForeignIssuedAccess,
                )
            },
        )
        .map(Some)
    }
}
