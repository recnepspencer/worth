//! A separate world whose one operation declares a managed-index decision
//! read, so the decision-read entry's own branch is reachable. Only the
//! decision-entry forgery court installs it.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::context::court_doors_for_test::ContextDoorsOps;

pub use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::operation::DecisionReadBinding;
pub use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::operation::DecisionReadOperation;
pub use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::schema::DecisionReadSchema;
pub use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::schema::Principal;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::external_identity;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest;
use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationPrincipalKey;
use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::facade::domain_computation::WorthQuerySelectedProductOperation;
use std::time::Duration;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_installation::facade::WorthQueryInstallationAdmissionProfile;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
use worth_query_installation::facade::WorthQueryInstalledPrincipalBinding;
use worth_query_installation::facade::WorthQueryPortableDomainIdentity;
use worth_query_installation::facade::WorthQueryPortableDomainPackage;

pub type DecisionReadAdmission = WorthQueryAdmittedApplicationOperation<
    DecisionReadSchema,
    DecisionReadOperation,
    (),
    Principal,
>;
pub type Binding = WorthQueryInstalledPrincipalBinding<
    DecisionReadSchema,
    worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::schema::DecisionPrincipalBinding,
    worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::schema::Mapping,
    Principal,
    u64,
    U64ApplicationValueBinding,
>;

/// The world's one bound principal, which is also its scope.
pub const PRINCIPAL: u64 = 1;

pub struct DecisionReadWorld {
    pub application: WorthQueryPrimaryGraphApplicationRuntime<DecisionReadSchema>,
    pub binding: Binding,
}

pub fn installed_decision_read_world() -> DecisionReadWorld {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|phase| {
        let declaration = DecisionReadSchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            <DecisionReadSchema as ApplicationSchema>::OWNER,
            1,
            0,
        ))
        .application_schema(declaration.clone())
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
        let (runtime, authority) = WorthQueryExecutionRuntimeInstaller::new()
            .install(WorthQueryInstallationGeneration::initial(), [admitted])
            .unwrap()
            .into_parts();
        let schema = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .unwrap();
        let binding = schema
            .principal_binding(worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::schema::DecisionPrincipalBinding::reference())
            .unwrap();
        let mut bootstrap = authority
            .prepare_primary_graph(
                &phase.bootstrap_for_test(),
                &runtime,
                &schema,
                worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()),
            )
            .unwrap();
        bootstrap
            .bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("principal-0").unwrap(),
                PRINCIPAL,
                external_identity("alice"),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .unwrap();
        (*bootstrap
            .managed_indexes_mut_for_test())
            .register_for_test::<worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::queries::DecisionIndex, _>(
                &schema,
                worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::schema::MEMBERS,
                worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::provider::DecisionIndexProvider,
                None,
            )
            .unwrap();
        let decision = schema
            .installed_mutation_binding::<DecisionReadBinding>()
            .unwrap();
        bootstrap
            .install_handler(&decision, worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::operation::DecisionReadHandler)
            .unwrap();
        let application = bootstrap
            .publish_application_runtime(
                &phase.bootstrap_for_test(),
                runtime,
                authority,
                schema,
                worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
            )
            .unwrap();
        DecisionReadWorld {
            application,
            binding,
        }
    })
}

impl DecisionReadWorld {
    pub fn authenticate(
        &self,
        request: &WorthQueryRequestScope,
    ) -> WorthQueryAuthenticatedExternalPrincipal<DecisionReadSchema> {
        authenticate_external(
            self.application.installed_schema(),
            "alice",
            Duration::from_secs(60),
            request,
        )
    }

    /// Mints the decision-read operation for the bound principal on its own scope.
    pub fn admit(&self, request: &WorthQueryRequestScope) -> DecisionReadAdmission {
        let operation = self
            .application
            .installed_schema()
            .installed_operation(DecisionReadOperation::reference())
            .unwrap();
        self.application
            .with_host_advancement(|phase| {
                let host = phase.test_host_request_for_test();
                let (principal, scope) = self.party(&host, request);
                self.selected(&host)
                    .authorize_operation(
                        &phase,
                        &principal,
                        &scope,
                        &operation,
                        Default::default(),
                        request,
                    )
                    .unwrap()
            })
            .expect("the fixture owns its installed host call")
    }

    pub fn party(
        &self,
        host: &TestHostRequest<'_>,
        request: &WorthQueryRequestScope,
    ) -> (
        WorthQueryAuthenticatedPrincipal<DecisionReadSchema, Principal, u64>,
        WorthQueryApplicationEntityIdentity<DecisionReadSchema, Principal>,
    ) {
        let phase = host.phase_for_application_for_test(&self.application);
        let principal = self
            .selected(host)
            .resolve_authenticated_principal(
                &phase,
                &self.binding,
                &self.authenticate(request),
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let scope = self
            .selected(host)
            .resolve_entity(
                &phase,
                worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::schema::PrincipalIdentity::reference(),
                PRINCIPAL,
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        (principal, scope)
    }

    pub fn selected(
        &self,
        host: &TestHostRequest<'_>,
    ) -> WorthQuerySelectedProductOperation<'_, DecisionReadSchema> {
        let execution = host
            .execution_request_for_for_test(self.application.product_runtime())
            .expect("the fixture lends its installed phase");
        self.application
            .select_product_branch_for_test(
                &execution,
                self.application.product_runtime().default_branch_for_test(),
            )
            .expect("the fixture product branch remains admitted")
    }
}
