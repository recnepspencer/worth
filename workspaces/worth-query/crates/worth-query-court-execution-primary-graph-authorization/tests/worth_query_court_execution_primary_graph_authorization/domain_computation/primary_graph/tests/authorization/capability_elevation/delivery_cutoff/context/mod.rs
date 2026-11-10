use std::num::NonZeroUsize;

use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_elevated_capability_live_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_elevated_capability_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_elevation_seed::CapabilityElevationScenario;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ElevatedAccountActivityQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::account_activity_query::ElevatedAccountActivityResult;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApprovedElevation;
use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub type InstalledQuery = WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    ElevatedAccountActivityQuery,
    AccountSummaryParameters,
    ElevatedAccountActivityResult,
    Account,
>;

pub type Authenticated = WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>;
pub type AccountScope = WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>;

pub struct ElevatedQueryContext {
    pub world: AuthorizationWorld,
    pub request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    pub approved: Option<WorthQueryApprovedElevation>,
    pub principal: Authenticated,
    pub committer: Authenticated,
    pub account: AccountScope,
    pub query: InstalledQuery,
}

impl ElevatedQueryContext {
    pub fn script_current_time(&mut self) {
        self.world
            .authorization_time
            .script(std::iter::repeat_n(time(100), 32));
    }

    pub fn elevated_access(
        &self,
        phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    ) -> worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess<
        IdentityExecutionSchema,
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::ElevatedTouchAccountCapability,
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::ElevatedCapabilityTouchOperation,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::ElevatedCapabilityTouchInput,
    >{
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::admit(
            &(phase_custody).test_host_request_for_test(),
            &self.world,
            self.approved.as_ref().unwrap(),
            &self.principal,
            &self.request,
            Some("elevation-2"),
        )
        .unwrap()
    }
}

pub fn context(live: bool) -> ElevatedQueryContext {
    let world = if live {
        installed_elevated_capability_live_world(CapabilityElevationScenario::Active)
    } else {
        installed_elevated_capability_world(CapabilityElevationScenario::Active)
    };
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 64));
    let request = live_scope();
    let requested = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::request_support::commit_exact_request(&world, &request);
    crate::domain_computation::primary_graph::tests::authorization::capability_elevation::request_support::resolve_exact_request_identities(&world, &request);
    let approved =
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::approve_exact_request(&world, &request, requested);
    let (principal, committer, account) = world
        .application
        .with_application_advancement(&request, |phase| {
            let custody = phase.test_host_request_for_test();
            let phase_custody = &custody;
            let borrowed_execution = phase
                .execution_request_for_for_test((*world.application.runtime_for_test()).product_runtime_for_test())
                .expect("the context read belongs to its installed world");
            let execution = &borrowed_execution;
            let principal =
                authenticated_principal(&(phase_custody).test_host_request_for_test(), &world, &request);
            let committer = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::authenticated(
                phase_custody,
                &world,
                "carol",
                &request,
            );
            let account = world
                .application
                .select_product_branch_for_test(
                    execution,
                    world.application.product_runtime().default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase_custody.phase_for_application_for_test(&world.application),
                    AccountIdentity::at("AccountIdentity"),
                    "account-1".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            (principal, committer, account)
        })
        .expect("the context read owns its host call");
    let query = world
        .application
        .installed_schema()
        .certification_query(ElevatedAccountActivityQuery::reference())
        .unwrap();
    ElevatedQueryContext {
        world,
        request,
        approved: Some(approved),
        principal,
        committer,
        account,
        query,
    }
}

pub fn assert_resources_released(world: &AuthorizationWorld) {
    assert_eq!(
        world
            .application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );

    assert_eq!(
        world.application.provider_session_resource_count_for_test(),
        0
    );
}

pub fn one() -> NonZeroUsize {
    NonZeroUsize::new(1).unwrap()
}
