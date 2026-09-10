use std::time::Duration;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};

use super::{authenticate_external, AuthorizationWorld, IdentityExecutionSchema, IdentityWorld};

impl IdentityWorld {
    pub(in crate::domain_computation::primary_graph::tests) fn selected_product(
        &self,
    ) -> crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
        '_,
        IdentityExecutionSchema,
    > {
        self.application
            .select_product_branch(self.application.product_runtime().default_branch())
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
