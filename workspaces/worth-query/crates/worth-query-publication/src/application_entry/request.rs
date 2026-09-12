use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
use worth_query_declaration::facade::application_query::ApplicationQueryIntent;
use worth_query_installation::facade::ApplicationSchema;

use super::{WorthQueryApplicationMutationRequest, WorthQueryApplicationQueryRequest};
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// Borrowed ordinary-request context. Construction selects no World state and
/// resolves no application principal.
pub struct WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema> {
    pub(super) application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) scope: &'scope WorthQueryRequestScope,
}

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
        }
    }
}

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn query<Intent>(
        &self,
        intent: Intent,
    ) -> WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent>
    where
        Intent: ApplicationQueryIntent<Schema>,
    {
        WorthQueryApplicationQueryRequest::new(self.application, self.principal, self.scope, intent)
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
            intent,
        )
    }
}
