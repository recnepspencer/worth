use std::num::NonZeroUsize;

use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::super::super::super::{
    application_attempt::authenticated_principal,
    fixture::{
        installed_elevated_capability_live_world, installed_elevated_capability_world, live_scope,
        Account, AccountIdentity, AccountSummaryParameters, AuthorizationWorld,
        CapabilityElevationScenario, ElevatedAccountActivityQuery, ElevatedAccountActivityResult,
        IdentityExecutionSchema, Principal,
    },
};
use super::super::super::capability_progression::time;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryApprovedElevation,
    WorthQueryAuthenticatedPrincipal, WorthQueryPrincipalResolutionMode,
};

pub(super) type InstalledQuery = WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    ElevatedAccountActivityQuery,
    AccountSummaryParameters,
    ElevatedAccountActivityResult,
    Account,
>;

type Authenticated = WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>;
type AccountScope = WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>;

pub(super) struct ElevatedQueryContext {
    pub(super) world: AuthorizationWorld,
    pub(super) request:
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    pub(super) approved: Option<WorthQueryApprovedElevation>,
    pub(super) principal: Authenticated,
    pub(super) committer: Authenticated,
    pub(super) account: AccountScope,
    pub(super) query: InstalledQuery,
}

impl ElevatedQueryContext {
    pub(super) fn script_current_time(&mut self) {
        self.world
            .authorization_time
            .script(std::iter::repeat_n(time(100), 32));
    }

    pub(super) fn elevated_access(
        &self,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess<
        IdentityExecutionSchema,
        super::super::ElevatedTouchAccountCapability,
        super::super::ElevatedCapabilityTouchOperation,
        super::super::ElevatedCapabilityTouchInput,
    > {
        super::super::admit(
            &self.world,
            self.approved.as_ref().unwrap(),
            &self.principal,
            &self.request,
            Some("elevation-2"),
        )
        .unwrap()
    }
}

pub(super) fn context(live: bool) -> ElevatedQueryContext {
    let world = if live {
        installed_elevated_capability_live_world(CapabilityElevationScenario::Active)
    } else {
        installed_elevated_capability_world(CapabilityElevationScenario::Active)
    };
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 64));
    let request = live_scope();
    let requested = super::super::request_support::commit_exact_request(&world, &request);
    super::super::request_support::resolve_exact_request_identities(&world, &request);
    let approved =
        super::super::approval_transition::approve_exact_request(&world, &request, requested);
    let principal = authenticated_principal(&world, &request);
    let committer = super::super::approval_transition::authenticated(&world, "carol", &request);
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

pub(super) fn assert_resources_released(world: &AuthorizationWorld) {
    assert_eq!(
        world
            .application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
    let buffers = world.application.result_buffer_observer().observe();
    assert_eq!(buffers.active_buffers(), 0);
    assert_eq!(buffers.retained_bytes(), 0);
    assert_eq!(world.application.provider_session_resource_count(), 0);
}

pub(super) fn one() -> NonZeroUsize {
    NonZeroUsize::new(1).unwrap()
}

pub(super) fn buffer_limit() -> NonZeroUsize {
    NonZeroUsize::new(4_096).unwrap()
}
