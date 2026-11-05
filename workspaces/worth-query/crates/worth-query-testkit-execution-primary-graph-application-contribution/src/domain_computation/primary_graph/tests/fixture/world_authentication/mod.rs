use std::time::Duration;

use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use crate::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external;
use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::IdentityWorld;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;

impl IdentityWorld {
    pub fn selected_product(
        &self,
        phase: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    ) -> worth_query_execution::facade::domain_computation::WorthQuerySelectedProductOperation<
        '_,
        IdentityExecutionSchema,
    > {
        let execution = phase
            .execution_request_for_for_test(self.application.product_runtime())
            .expect("fixture lends its installed phase");
        self.application
            .select_product_branch_for_test(
                &execution,
                self.application.product_runtime().default_branch_for_test(),
            )
            .expect("the fixture product branch remains admitted")
    }

    pub fn authenticate(
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
    pub fn authenticate(
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
