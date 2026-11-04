use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatusBinding;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryContinuation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationContinuationPageResult;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryResumeControls;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::live_account_parameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::LiveAccountActivityQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::LiveAccountActivityResult;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub type TestContinuation = WorthQueryApplicationQueryContinuation<
    IdentityExecutionSchema,
    LiveAccountActivityQuery,
    AccountSummaryParameters,
    LiveAccountActivityResult,
    Account,
>;

pub struct ContinuationTestContext {
    pub world: AuthorizationWorld,
    pub principal: WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    pub account: WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    pub query: WorthQueryInstalledApplicationQuery<
        IdentityExecutionSchema,
        LiveAccountActivityQuery,
        AccountSummaryParameters,
        LiveAccountActivityResult,
        Account,
    >,
}

impl ContinuationTestContext {
    pub fn new(
        authentication_lifetime: Duration,
        clock: worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock,
    ) -> Self {
        let world = installed_authorization_world(true, clock.clone());
        let request = worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(Some(clock.now() + Duration::from_secs(60)), worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new().token(), clock.clone());
        let external = world.authenticate("alice", authentication_lifetime, &request);
        let (principal, account, query) = world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase;
                let borrowed_execution = phase
                    .execution_request_for_for_test(world.application.product_runtime())
                    .expect("the installed fixture lends its own phase");
                let execution = &borrowed_execution;
                let principal = world
                    .application
                    .select_product_branch_for_test(
                        execution,
                        world
                            .application
                            .product_runtime()
                            .default_branch_for_test(),
                    )
                    .expect("the selected product branch remains admitted")
                    .resolve_authenticated_principal(
                        phase_custody,
                        &world.binding,
                        &external,
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let account = world
                    .application
                    .select_product_branch_for_test(
                        execution,
                        world
                            .application
                            .product_runtime()
                            .default_branch_for_test(),
                    )
                    .expect("the selected product branch remains admitted")
                    .resolve_entity(
                        phase_custody,
                        AccountIdentity::reference(),
                        "account-1".to_owned(),
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let query = world
                    .application
                    .installed_schema()
                    .certification_query(LiveAccountActivityQuery::reference())
                    .unwrap();

                (principal, account, query)
            })
            .expect("the fixture host admits its identity reads");
        Self {
            world,
            principal,
            account,
            query,
        }
    }

    pub fn issue(
        &self,
        phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    ) -> TestContinuation {
        let request = worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(Some(self.principal.clock().now() + Duration::from_secs(60)), worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new().token(), self.principal.clock().clone());
        let access = WorthQueryApplicationQueryAccessContext::new(&self.principal, &self.account);
        let plan = self
            .world
            .selected_product(phase_custody)
            .admit_application_query_continuation(
                &phase_custody.phase_for_application_for_test(&self.world.application),
                &self.query,
                &access,
                parameters("account-1"),
                worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
                    NonZeroUsize::new(1).unwrap(),
                    &request,
                ),
            )
            .unwrap();
        let page: WorthQueryApplicationContinuationPageResult<
            IdentityExecutionSchema,
            LiveAccountActivityQuery,
            AccountSummaryParameters,
            LiveAccountActivityResult,
            Account,
        > = self
            .world
            .application
            .execute_application_query_continuation_page(
                &phase_custody.phase_for_application_for_test(&self.world.application),
                plan,
            )
            .unwrap();
        let (_, continuation, receipt) = page.into_parts();
        assert!(receipt.basis_released());
        let continuation = continuation.expect("the two-row fixture must continue");
        self.assert_resource_baseline();
        continuation
    }

    pub fn readmit_denial(
        &self,
        phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
        continuation: TestContinuation,
        parameter: &str,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        page_width: usize,
    ) -> WorthQueryApplicationQueryAdmissionDenialKind {
        let access = WorthQueryApplicationQueryAccessContext::new(&self.principal, &self.account);
        self.world
            .application
            .readmit_application_query_continuation(
                &phase_custody.phase_for_application_for_test(&self.world.application),
                &self.query,
                &access,
                parameters(parameter),
                continuation,
                WorthQueryApplicationQueryResumeControls::new(
                    NonZeroUsize::new(page_width).unwrap(),
                    request,
                ),
            )
            .err()
            .expect("the hostile continuation must deny")
            .kind()
    }

    /// Resumes and executes one page; returns its row count and basis release.
    pub fn readmit_page(
        &self,
        phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
        continuation: TestContinuation,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        page_width: usize,
    ) -> (usize, bool) {
        let access = WorthQueryApplicationQueryAccessContext::new(&self.principal, &self.account);
        let phase = phase_custody.phase_for_application_for_test(&self.world.application);
        let plan = self
            .world
            .application
            .readmit_application_query_continuation(
                &phase,
                &self.query,
                &access,
                parameters("account-1"),
                continuation,
                WorthQueryApplicationQueryResumeControls::new(
                    NonZeroUsize::new(page_width).unwrap(),
                    request,
                ),
            )
            .expect("a resume carries no work forecast to exhaust");
        let page = self
            .world
            .application
            .execute_application_query_continuation_page(&phase, plan)
            .expect("the admitted resume reads its page");
        (page.rows().len(), page.receipt().basis_released())
    }

    pub fn basis_acquisitions(&self) -> usize {
        self.world
            .application
            .application_query_basis_observer()
            .observe()
            .acquisitions()
    }

    pub fn assert_resource_baseline(&self) {
        let basis = self
            .world
            .application
            .application_query_basis_observer()
            .observe();
        assert_eq!(basis.active(), 0);

        assert_eq!(
            self.world
                .application
                .provider_session_resource_count_for_test(),
            0
        );
    }

    pub fn advance_installation(&mut self) {
        let successor = Arc::new(
            (*self.world.application.runtime_for_test())
                .installed_packages_for_test()
                .successor_generation(),
        );
        (*self.world.application.runtime_mut_for_test())
            .execution_mut_for_test()
            .commit_successor_installation(successor)
            .unwrap();
    }

    pub fn stale_principal_mapping(&self, execution: &worth_execution::ExecutionRequest<'_, '_>) {
        let graph = (*self.world.application.runtime_for_test())
            .primary_graph_for_test()
            .expect("test world publishes a primary graph");
        let layout = (*graph.layout_for_test())
            .principal_binding_for_test(self.principal.binding())
            .expect("test principal binding is installed")
            .clone();
        mutate_field(
            execution,
            &self.world,
            self.principal.mapping_entity_id_for_test(),
            layout.status_locator_into_for_test(),
            WorthQueryPrincipalMappingStatusBinding::encode(
                &WorthQueryPrincipalMappingStatus::Disabled,
            )
            .unwrap(),
            "stale-continuation-principal",
        );
    }

    pub fn stale_scope_identity(&self, execution: &worth_execution::ExecutionRequest<'_, '_>) {
        let graph = (*self.world.application.runtime_for_test())
            .primary_graph_for_test()
            .expect("test world publishes a primary graph");
        let field = AccountIdentity::reference();
        let locator = (*graph.layout_for_test())
            .field_locator_for_test(field.entity(), field.aspect(), field.field())
            .expect("account identity is installed")
            .clone();
        mutate_field(
            execution,
            &self.world,
            self.account.entity_id_for_test(),
            locator,
            StringApplicationValueBinding::encode(&"account-renamed".to_owned()).unwrap(),
            "stale-continuation-scope",
        );
    }
}

pub fn parameters(account: &str) -> ApplicationQueryParameterSet<LiveAccountActivityQuery> {
    live_account_parameters(account)
}

pub fn mutate_field(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    entity_id: worth_relational::facade::identity::EntityId,
    locator: worth_relational::facade::transactions::AspectFieldLocator,
    value: worth_foundational::facade::AspectValue,
    batch: &str,
) {
    let fields = AspectFieldPatch::from(BTreeMap::from([(locator, value)]));
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new(batch).push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }),
        )),
    );
}
