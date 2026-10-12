use std::time::Duration;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};

use super::{authenticate_external, AuthorizationWorld, IdentityExecutionSchema, IdentityWorld};

impl IdentityWorld {
    pub(in crate::domain_computation::primary_graph::tests) fn selected_product(
        &self,
        phase: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    ) -> crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
        '_,
        IdentityExecutionSchema,
    > {
        let execution = phase
            .execution_request_for(self.application.product_runtime())
            .expect("fixture lends its installed phase");
        self.application
            .select_product_branch(
                &execution,
                self.application.product_runtime().default_branch(),
            )
            .expect("the fixture product branch remains admitted")
    }

    pub(in crate::domain_computation::primary_graph::tests) fn authenticate(
        &self,
        subject: &str,
        lifetime: Duration,
        scope: &WorthQueryRequestScope,
    ) -> WorthQueryAuthenticatedExternalPrincipal<IdentityExecutionSchema> {
        authenticate_external(
            self.application.installed_schema(),
            subject,
            lifetime,
            scope,
        )
    }
}

impl AuthorizationWorld {
    pub(in crate::domain_computation::primary_graph) fn authenticate(
        &self,
        subject: &str,
        lifetime: Duration,
        scope: &WorthQueryRequestScope,
    ) -> WorthQueryAuthenticatedExternalPrincipal<IdentityExecutionSchema> {
        authenticate_external(
            self.application.installed_schema(),
            subject,
            lifetime,
            scope,
        )
    }
}
