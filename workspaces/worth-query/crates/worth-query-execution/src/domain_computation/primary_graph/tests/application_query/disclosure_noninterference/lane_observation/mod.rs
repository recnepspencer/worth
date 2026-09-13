use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::time::{Duration, UNIX_EPOCH};

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::{
    ApplicationScalarValueBinding, WorthQueryInstalledApplicationQuery,
};

use super::super::super::fixture::{
    installed_capability_live_world_with_label, live_scope, Account, AccountIdentity,
    AccountSummaryParameters, AuthorizationWorld, CapabilityDisclosure,
    GovernedLiveAccountActivityQuery, GovernedLiveAccountActivityResult, IdentityExecutionSchema,
    Principal,
};
use super::receipt_observation::{StableReadCompletionObservation, StableReceiptObservation};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDisclosed, WorthQueryApplicationEntityIdentity,
    WorthQueryApplicationQueryAccessReceipt, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrincipalResolutionMode,
};

mod live;
mod non_live;

type InstalledGovernedQuery = WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    GovernedLiveAccountActivityQuery,
    AccountSummaryParameters,
    GovernedLiveAccountActivityResult,
    Account,
>;

#[derive(Debug, Eq, PartialEq)]
struct ProtectedWorldObservation {
    one_shot: LaneObservation,
    continuation: ContinuationObservation,
    live: LaneObservation,
    live_commit_ordinal: u64,
    live_close: StableReadCompletionObservation,
}

#[derive(Debug, Eq, PartialEq)]
struct ContinuationObservation {
    first: LaneObservation,
    first_next_page_ordinal: u64,
    second: LaneObservation,
    second_has_continuation: bool,
}

#[derive(Debug, Eq, PartialEq)]
struct LaneObservation {
    rows: Vec<GovernedLiveAccountActivityResult>,
    receipt: StableReceiptObservation,
}

struct GovernedObservationContext<'a> {
    world: &'a AuthorizationWorld,
    request: &'a WorthQueryRequestScope,
    principal: &'a WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    committer: &'a WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: &'a WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    query: &'a InstalledGovernedQuery,
}

#[derive(Default)]
struct GraphWorkOccurrences {
    sessions: BTreeSet<u64>,
    managed_runs: BTreeSet<u64>,
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

fn observe_protected_world(label: &str) -> ProtectedWorldObservation {
    let world = installed_capability_live_world_with_label(label);
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    let request = live_scope();
    let principal = resolve_principal(&world, "alice", &request);
    let committer = resolve_principal(&world, "bob", &request);
    let account = resolve_account(&world, &request);
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

    let one_shot = non_live::observe_one_shot(&context, &mut occurrences);
    let continuation = non_live::observe_continuation(&context, &mut occurrences);
    let (live, live_commit_ordinal, live_close) = live::observe(&context, label, &mut occurrences);

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

fn capture_lane(
    rows: Vec<GovernedLiveAccountActivityResult>,
    receipt: &WorthQueryApplicationQueryAccessReceipt,
    occurrences: &mut GraphWorkOccurrences,
) -> LaneObservation {
    assert_eq!(rows.len(), receipt.result_count());
    assert!(receipt.basis_released());
    assert_eq!(receipt.disclosure().disclosure_decision_count(), 5);
    assert_eq!(receipt.disclosure().omitted().len(), 1);
    let buffer = receipt
        .result_buffer()
        .expect("every delivered lane is bounded");
    assert!(buffer.released());
    assert!(buffer.peak_bytes() <= buffer.limit_bytes());
    for row in &rows {
        assert_eq!(row.account(), "account-1");
        let WorthQueryApplicationDisclosed::Omitted(omission) = row.label() else {
            panic!("the protected label must already be a typed omission");
        };
        assert_eq!(omission.classification(), "account-activity");
        assert_eq!(
            omission.required_disclosure(),
            &super::super::super::fixture::CapabilityDisclosureBinding::encode(
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
    fn assert_same(&self, other: &Self) {
        assert_eq!(self.rows, other.rows, "projected rows");
        self.receipt.assert_same(&other.receipt);
    }
}

impl ContinuationObservation {
    fn assert_same(&self, other: &Self) {
        self.first.assert_same(&other.first);
        assert_eq!(self.first_next_page_ordinal, other.first_next_page_ordinal);
        self.second.assert_same(&other.second);
        assert_eq!(self.second_has_continuation, other.second_has_continuation);
    }
}

impl GraphWorkOccurrences {
    fn capture(
        &mut self,
        completion: &crate::domain_computation::provider_session::WorthQueryGraphReadCompletion,
    ) {
        assert!(self.sessions.insert(completion.session_identity().as_u64()));
        assert!(self
            .managed_runs
            .insert(completion.managed_run_identity().as_u64()));
    }
}

fn resolve_principal(
    world: &AuthorizationWorld,
    external_identity: &str,
    request: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64> {
    let external = world.authenticate(external_identity, Duration::from_secs(60), request);
    world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

fn resolve_account(
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
) -> WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account> {
    world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountIdentity::reference(),
            "account-1".to_owned(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

fn installed_governed_query(world: &AuthorizationWorld) -> InstalledGovernedQuery {
    world
        .application
        .installed_schema()
        .certification_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap()
}

fn assert_full_rows(rows: &[GovernedLiveAccountActivityResult]) {
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].activities(),
        &[
            ("activity-primary".to_owned(), 11),
            ("activity-secondary".to_owned(), 22),
        ]
    );
}

fn one() -> NonZeroUsize {
    NonZeroUsize::new(1).unwrap()
}

fn buffer_limit() -> NonZeroUsize {
    NonZeroUsize::new(4_096).unwrap()
}
