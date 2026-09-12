use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent,
};
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;

pub struct WorthQueryApplicationMutationRequest<'application, 'principal, 'scope, Schema, Intent> {
    pub(super) application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) scope: &'scope WorthQueryRequestScope,
    pub(super) intent: Intent,
}

pub struct WorthQueryApplicationMutationRequestWithIdempotency<
    'application,
    'principal,
    'scope,
    'key,
    Schema,
    Intent,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(super) request:
        WorthQueryApplicationMutationRequest<'application, 'principal, 'scope, Schema, Intent>,
    pub(super) key: &'key <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
}

impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationMutationRequest<'application, 'principal, 'scope, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(in crate::application_entry) const fn new(
        application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &'principal WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &'scope WorthQueryRequestScope,
        intent: Intent,
    ) -> Self {
        Self {
            application,
            principal,
            scope,
            intent,
        }
    }

    pub fn idempotency<'key>(
        self,
        key: &'key <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    ) -> WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
    > {
        WorthQueryApplicationMutationRequestWithIdempotency { request: self, key }
    }
}
