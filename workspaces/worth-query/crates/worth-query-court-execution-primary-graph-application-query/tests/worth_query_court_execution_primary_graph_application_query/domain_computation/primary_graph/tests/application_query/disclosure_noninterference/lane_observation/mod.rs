use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::context::court_doors_for_test::ContextDoorsOps;
use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_live_world_with_label;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedLiveAccountActivityQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::GovernedLiveAccountActivityResult;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::receipt_observation::StableReadCompletionObservation;
use crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::receipt_observation::StableReceiptObservation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationDisclosed;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessReceipt;
use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub type InstalledGovernedQuery = WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    GovernedLiveAccountActivityQuery,
    AccountSummaryParameters,
    GovernedLiveAccountActivityResult,
    Account,
>;

#[derive(Debug, Eq, PartialEq)]
pub struct ProtectedWorldObservation {
    pub one_shot: LaneObservation,
    pub continuation: ContinuationObservation,
    pub live: LaneObservation,
    pub live_commit_ordinal: u64,
    pub live_close: StableReadCompletionObservation,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ContinuationObservation {
    pub first: LaneObservation,
    pub first_next_page_ordinal: u64,
    pub second: LaneObservation,
    pub second_has_continuation: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct LaneObservation {
    pub rows: Vec<GovernedLiveAccountActivityResult>,
    pub receipt: StableReceiptObservation,
}

pub struct GovernedObservationContext<'a> {
    pub world: &'a AuthorizationWorld,
    pub request: &'a WorthQueryRequestScope,
    pub principal: &'a WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    pub committer: &'a WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    pub account: &'a WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    pub query: &'a InstalledGovernedQuery,
}

#[derive(Default)]
pub struct GraphWorkOccurrences {
    pub sessions: BTreeSet<u64>,
    pub managed_runs: BTreeSet<u64>,
}

#[test]
fn protected_label_is_absent_from_every_implemented_lane_observable() {
    let left = observe_protected_world("x");
    let right = observe_protected_world(
        "a-protected-label-whose-length-would-expose-buffer-accounting-leakage",
    );

    left.one_shot.assert_same(&right.one_shot);
    left.continuation.assert_same(&right.continuation);
    left.live.assert_same(&right.live);
    assert_eq!(left.live_commit_ordinal, right.live_commit_ordinal);
    assert_eq!(left.live_close, right.live_close, "live close observable");
    assert_full_rows(&left.one_shot.rows);
    assert_eq!(
        left.continuation.first.rows[0].activities(),
        &[("activity-primary".to_owned(), 11)]
    );
    assert_eq!(
        left.continuation.second.rows[0].activities(),
        &[("activity-secondary".to_owned(), 22)]
    );
    assert_eq!(left.continuation.first_next_page_ordinal, 2);
    assert!(!left.continuation.second_has_continuation);
    assert_eq!(
        left.live.rows[0].activities(),
        &[("activity-primary".to_owned(), 11)]
    );
}

pub fn observe_protected_world(label: &str) -> ProtectedWorldObservation {
    let world = installed_capability_live_world_with_label(label);
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    let request = live_scope();
    let (principal, committer, account) = world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request_for_test();
            let phase_custody = &custody;
            let principal = resolve_principal(
                phase_custody,
                &(phase_custody).borrowed_request_for_test(),
                &world,
                "alice",
                &request,
            );
            let committer = resolve_principal(
                phase_custody,
                &(phase_custody).borrowed_request_for_test(),
                &world,
                "bob",
                &request,
            );
            let account = resolve_account(
                phase_custody,
                &phase_custody.borrowed_request_for_test(),
                &world,
                &request,
            );
            (principal, committer, account)
        })
        .expect("the disclosure fixture owns its identity call");
    let query = installed_governed_query(&world);
    let context = GovernedObservationContext {
        world: &world,
        request: &request,
        principal: &principal,
        committer: &committer,
        account: &account,
        query: &query,
    };
    let mut occurrences = GraphWorkOccurrences::default();

    let one_shot = world
        .application
        .with_host_advancement(|phase| {
            crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::non_live::observe_one_shot(&phase.test_host_request_for_test(), &context, &mut occurrences)
        })
        .expect("the fixture owns its one-shot call");
    let continuation = world
        .application
        .with_host_advancement(|phase| {
            crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::non_live::observe_continuation(&phase.test_host_request_for_test(), &context, &mut occurrences)
        })
        .expect("the fixture owns its continuation call");
    let (live, live_commit_ordinal, live_close) = crate::domain_computation::primary_graph::tests::application_query::disclosure_noninterference::lane_observation::live::observe(&context, label, &mut occurrences);

    assert_eq!(occurrences.sessions.len(), 5);
    assert_eq!(occurrences.managed_runs.len(), 5);
    ProtectedWorldObservation {
        one_shot,
        continuation,
        live,
        live_commit_ordinal,
        live_close,
    }
}

pub fn capture_lane(
    rows: Vec<GovernedLiveAccountActivityResult>,
    receipt: &WorthQueryApplicationQueryAccessReceipt,
    occurrences: &mut GraphWorkOccurrences,
) -> LaneObservation {
    assert_eq!(rows.len(), receipt.result_count());
    assert!(receipt.basis_released());
    assert_eq!(receipt.disclosure().disclosure_decision_count(), 5);
    assert_eq!(receipt.disclosure().omitted().len(), 1);

    for row in &rows {
        assert_eq!(row.account(), "account-1");
        let WorthQueryApplicationDisclosed::Omitted(omission) = row.label() else {
            panic!("the protected label must already be a typed omission");
        };
        assert_eq!(omission.classification(), "account-activity");
        assert_eq!(
            omission.required_disclosure(),
            &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityDisclosureBinding::encode(
                &CapabilityDisclosure::PrivateLabel,
            )
            .unwrap()
        );
    }
    occurrences.capture(receipt.read_completion());
    LaneObservation {
        rows,
        receipt: StableReceiptObservation::capture(receipt),
    }
}

impl LaneObservation {
    pub fn assert_same(&self, other: &Self) {
        assert_eq!(self.rows, other.rows, "projected rows");
        self.receipt.assert_same(&other.receipt);
    }
}

impl ContinuationObservation {
    pub fn assert_same(&self, other: &Self) {
        self.first.assert_same(&other.first);
        assert_eq!(self.first_next_page_ordinal, other.first_next_page_ordinal);
        self.second.assert_same(&other.second);
        assert_eq!(self.second_has_continuation, other.second_has_continuation);
    }
}

impl GraphWorkOccurrences {
    pub fn capture(
        &mut self,
        completion: &worth_query_execution::facade::domain_computation::WorthQueryGraphReadCompletion,
    ) {
        assert!(self.sessions.insert(completion.session_identity().as_u64()));
        assert!(self
            .managed_runs
            .insert(completion.managed_run_identity().as_u64()));
    }
}

pub fn resolve_principal(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    external_identity: &str,
    request: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64> {
    let external = world.authenticate(external_identity, Duration::from_secs(60), request);
    world
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
            &phase_custody.phase_for_application_for_test(&world.application),
            &world.binding,
            &external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

pub fn resolve_account(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
) -> WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account> {
    world
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
            &phase_custody.phase_for_application_for_test(&world.application),
            AccountIdentity::reference(),
            "account-1".to_owned(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

pub fn installed_governed_query(world: &AuthorizationWorld) -> InstalledGovernedQuery {
    world
        .application
        .installed_schema()
        .certification_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap()
}

pub fn assert_full_rows(rows: &[GovernedLiveAccountActivityResult]) {
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].activities(),
        &[
            ("activity-primary".to_owned(), 11),
            ("activity-secondary".to_owned(), 22),
        ]
    );
}

pub fn one() -> NonZeroUsize {
    NonZeroUsize::new(1).unwrap()
}

#[path = "live/mod.rs"]
pub mod live;
#[path = "non_live/mod.rs"]
pub mod non_live;
