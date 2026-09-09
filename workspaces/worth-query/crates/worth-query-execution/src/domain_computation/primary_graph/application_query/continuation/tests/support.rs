use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_installation::facade::{
    TypedApplicationValue, WorthQueryInstalledApplicationQuery,
};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::super::authority::WorthQueryApplicationQueryContinuation;
use crate::domain_computation::primary_graph::{
    application_query::{
        WorthQueryApplicationContinuationPageResult, WorthQueryApplicationQueryAccessContext,
        WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryApplicationQueryResumeControls,
    },
    tests::fixture::{
        installed_authorization_world, live_account_parameters, Account, AccountIdentity,
        AccountSummaryParameters, AuthorizationWorld, IdentityExecutionSchema,
        LiveAccountActivityQuery, LiveAccountActivityResult, Principal,
    },
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrincipalResolutionMode,
};

pub(super) type TestContinuation = WorthQueryApplicationQueryContinuation<
    IdentityExecutionSchema,
    LiveAccountActivityQuery,
    AccountSummaryParameters,
    LiveAccountActivityResult,
    Account,
>;

pub(super) struct ContinuationTestContext {
    pub(super) world: AuthorizationWorld,
    principal: WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    query: WorthQueryInstalledApplicationQuery<
        IdentityExecutionSchema,
        LiveAccountActivityQuery,
        AccountSummaryParameters,
        LiveAccountActivityResult,
        Account,
    >,
}

impl ContinuationTestContext {
    pub(super) fn new(authentication_lifetime: Duration) -> Self {
        let world = installed_authorization_world(true);
        let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
        let external = world.authenticate("alice", authentication_lifetime, &request);
        let principal = world
            .application
            .select_product_branch(world.application.product_runtime().default_branch())
            .expect("the selected product branch remains admitted")
            .resolve_authenticated_principal(
                &world.binding,
                external,
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let account = world
            .application
            .select_product_branch(world.application.product_runtime().default_branch())
            .expect("the selected product branch remains admitted")
            .resolve_entity(
                AccountIdentity::reference(),
                "account-1".to_owned(),
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let query = world
            .application
            .installed_schema()
            .application_query(LiveAccountActivityQuery::reference())
            .unwrap();
        Self {
            world,
            principal,
            account,
            query,
        }
    }

    pub(super) fn issue(&self) -> TestContinuation {
        let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
        let access = WorthQueryApplicationQueryAccessContext::new(&self.principal, &self.account);
        let plan = self
            .world
            .selected_product()
            .admit_application_query_continuation(
                &self.query,
                &access,
                parameters("account-1"),
                crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(10_000).unwrap(),
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
            .execute_application_query_continuation_page(plan)
            .unwrap();
        let (_, continuation, receipt) = page.into_parts();
        assert!(receipt.basis_released());
        let continuation = continuation.expect("the two-row fixture must continue");
        self.assert_resource_baseline();
        continuation
    }

    pub(super) fn readmit_denial(
        &self,
        continuation: TestContinuation,
        parameter: &str,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        page_width: usize,
        maximum_work: usize,
    ) -> WorthQueryApplicationQueryAdmissionDenialKind {
        let access = WorthQueryApplicationQueryAccessContext::new(&self.principal, &self.account);
        self.world
            .application
            .readmit_application_query_continuation(
                &self.query,
                &access,
                parameters(parameter),
                continuation,
                WorthQueryApplicationQueryResumeControls::new(
                    NonZeroUsize::new(page_width).unwrap(),
                    NonZeroUsize::new(maximum_work).unwrap(),
                    request,
                ),
            )
            .err()
            .expect("the hostile continuation must deny")
            .kind()
    }

    pub(super) fn basis_acquisitions(&self) -> usize {
        self.world
            .application
            .application_query_basis_observer()
            .observe()
            .acquisitions()
    }

    pub(super) fn expire_authentication(&mut self) {
        self.world
            .application
            .fix_authentication_time(self.principal.valid_until());
    }

    pub(super) fn assert_resource_baseline(&self) {
        let basis = self
            .world
            .application
            .application_query_basis_observer()
            .observe();
        assert_eq!(basis.active(), 0);
        let buffers = self.world.application.result_buffer_observer().observe();
        assert_eq!(buffers.active_buffers(), 0);
        assert_eq!(buffers.retained_bytes(), 0);
        assert_eq!(self.world.application.provider_session_resource_count(), 0);
    }

    pub(super) fn advance_installation(&mut self) {
        let successor = Arc::new(
            self.world
                .application
                .runtime
                .installed_packages()
                .successor_generation(),
        );
        self.world
            .application
            .runtime
            .commit_successor_installation(successor)
            .unwrap();
    }

    pub(super) fn stale_principal_mapping(&self) {
        let graph = self
            .world
            .application
            .runtime
            .primary_graph()
            .expect("test world publishes a primary graph");
        let layout = graph
            .layout
            .principal_binding(self.principal.binding())
            .expect("test principal binding is installed")
            .clone();
        mutate_field(
            &self.world,
            self.principal.mapping_entity_id(),
            layout.status_locator,
            WorthQueryPrincipalMappingStatus::Disabled.into_foundational_value(),
            "stale-continuation-principal",
        );
    }

    pub(super) fn stale_scope_identity(&self) {
        let graph = self
            .world
            .application
            .runtime
            .primary_graph()
            .expect("test world publishes a primary graph");
        let field = AccountIdentity::reference();
        let locator = graph
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .expect("account identity is installed")
            .clone();
        mutate_field(
            &self.world,
            self.account.entity_id(),
            locator,
            "account-renamed".to_owned().into_foundational_value(),
            "stale-continuation-scope",
        );
    }
}

fn parameters(account: &str) -> ApplicationQueryParameterSet<LiveAccountActivityQuery> {
    live_account_parameters(account)
}

fn mutate_field(
    world: &AuthorizationWorld,
    entity_id: worth_relational::facade::identity::EntityId,
    locator: worth_relational::facade::transactions::AspectFieldLocator,
    value: worth_foundational::facade::AspectValue,
    batch: &str,
) {
    let fields = AspectFieldPatch::from(BTreeMap::from([(locator, value)]));
    crate::domain_computation::primary_graph::tests::fixture::publish_relational_mutation(
        world,
        WorkerIntentBatch::new(batch).push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }),
        )),
    );
}
