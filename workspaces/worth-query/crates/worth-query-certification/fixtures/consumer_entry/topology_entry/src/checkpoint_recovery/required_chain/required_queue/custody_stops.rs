//! A caller refused required custody stops for itself, never for its rows.

use super::*;
use std::time::{Duration, Instant};
use support::capacity_region::Attempt;
use worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial;

macro_rules! start_root {
    ($application:expr, $request:expr) => {
        $request
            .demand(PlanarOutputDemand::new("anchor-a"))
            .start_in_program::<program::ChainProgram, program::ChainRoot>(&$application)
            .unwrap()
    };
}

macro_rules! start_consumer {
    ($application:expr, $request:expr, $scope:literal) => {
        $request
            .demand(ChainDemand($scope.to_owned()))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                &$application,
            )
            .unwrap()
    };
}

/// A, B and C form the chain, settled; no other demand is open.
macro_rules! chain {
    ($application:expr, $request:expr) => {
        chain!($application, $request, (), ())
    };
    ($application:expr, $request:expr, $before:expr, $after:expr) => {{
        let mut a = start_root!($application, $request);
        let mut b = start_consumer!($application, $request, "anchor-b");
        let mut c = start_consumer!($application, $request, "anchor-c");
        $before;
        settle!(a, $request);
        settle!(b, $request);
        settle!(c, $request);
        $after;
        (a, b, c)
    }};
}

#[test]
fn a_lone_caller_whose_chain_outgrows_its_budget_stops_without_retry() {
    let _guard = checkpoint_recovery_test_guard();
    support::capacity_region::search(
        "lone terminal",
        1,
        128 * 1024,
        support::capacity_region::Goal::Hit,
        lone_terminal,
    )
    .require_hit("lone terminal");
}

fn lone_terminal(budget: usize) -> Attempt {
    // Dispatch fits beside the settled chain; its typed successor cannot fit.
    let (application, _invalidation) = limited_application(budget, 128 * 1_024 * 1_024, 8);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, mut c) = setup_chain!(application, request);
    let settled_custody = application.required_custody_bytes_for_test();
    change_root_input!(request, application, 2, 0x9176_4000_u64);
    // The last consumer's caller is the only open demand, so every row that
    // holds custody is one its own advance needs: no other demand's advance
    // or close frees any.
    drop((a, b));
    for attempt in 0..3 {
        let entries = query_entries();
        let stopped = c.advance(&request);
        if attempt == 0 {
            match &stopped {
                Ok(_) => return Attempt::Above("lone refresh admitted"),
                Err(WorthQueryApplicationOutputDemandDenial::Demand(denial))
                    if denial.kind()
                        == WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
                        && denial.recovery_posture()
                            == primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable =>
                {
                    return Attempt::Below("upstream publication offers retry before terminal stop")
                }
                _ => (),
            }
        }
        assert!(
            matches!(
                &stopped,
                Err(WorthQueryApplicationOutputDemandDenial::Demand(denial))
                    if denial.kind() == WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
                        && denial.recovery_posture()
                            == primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal
            ),
            "attempt {attempt}: a retry that would meet the same custody is not offered: {:?}",
            stopped.as_ref().err()
        );
        // The stop is this caller's. Its row is not failed: an advance on it
        // runs the refresh again instead of answering with a kept failure.
        assert!(
            query_entries() > entries,
            "attempt {attempt}: the advance ran the chain's refresh again"
        );
        assert_eq!(
            application.required_custody_bytes_for_test(),
            settled_custody,
            "attempt {attempt}: the stop keeps what the settled chain held"
        );
    }
    // Nor are the rows it reads: demands that fit refresh them and settle.
    drop(c);
    let mut a = start_root!(application, request);
    settle!(a, request);
    let mut b = start_consumer!(application, request, "anchor-b");
    settle!(b, request);
    drop((a, b));
    Attempt::Hit
}

#[test]
fn a_refresh_cancelled_under_tight_custody_settles_on_the_next_advance() {
    let _guard = checkpoint_recovery_test_guard();
    let row = worth_query_host::facade::primary_graph::required_ready_custody_bytes_for_test();
    // Thirteen rows fund a settled chain beside one refresh.
    let (application, _invalidation) = limited_application(13 * row, 128 * 1_024 * 1_024, 8);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, mut c) = chain!(application, request);
    change_root_input!(request, application, 2, 0x9176_4100_u64);
    // The middle consumer's caller refreshes the root, then its request is
    // cancelled inside the middle row's refresh.
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let cancelled_scope = authentication::WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(120),
        cancellation.token(),
    );
    let cancelled = application.request(&principal, &cancelled_scope);
    take_decisions("anchor-b");
    super::super::binding::cancel_during_next_decision("anchor-b", cancellation);
    let stopped = b.advance(&cancelled);
    assert!(
        matches!(
            &stopped,
            Err(WorthQueryApplicationOutputDemandDenial::Demand(denial))
                if denial.kind() == WorthQueryOutputDemandDenialKind::Cancelled
        ),
        "the cancelled request meets its own stop: {:?}",
        stopped.as_ref().err()
    );
    assert_eq!(
        take_decisions("anchor-b").len(),
        1,
        "the request was cancelled inside the middle row's refresh"
    );
    // The interrupted refresh gives its reservation back before the one that
    // replaces it reserves: the same caller's next advance settles.
    let settled = settled_in_one_advance!(b, request, "the cancelled caller");
    assert_eq!(
        settled.producer_contacts_in_this_demand(),
        3,
        "initial, cancelled refresh, and successful retry all entered B's handler"
    );
    let again = settled_in_one_advance!(b, request, "the settled caller advanced again");
    assert_eq!(
        again.producer_contacts_in_this_demand(),
        3,
        "a handle reports its lifetime count"
    );
    drop((settled, again));
    settled_in_one_advance!(c, request, "the last consumer");
    settled_in_one_advance!(a, request, "the open root demand");
    drop((a, b, c));
}

/// Settles within eight advances. Every stop on the way offers a retry: any
/// other stop fails here, and so does a demand that defers on every advance.
macro_rules! settled_within_eight_advances {
    ($demand:expr, $request:expr, $what:expr) => {{
        let mut stops = Vec::new();
        (0..8)
            .find_map(|_| match $demand.advance(&$request) {
                Ok(WorthQueryApplicationOutputDemandProgress::Settled(settled)) => Some(settled),
                Ok(WorthQueryApplicationOutputDemandProgress::Pending) => None,
                Err(stop) => {
                    assert!(
                        matches!(
                            &stop,
                            WorthQueryApplicationOutputDemandDenial::Demand(denial)
                                if denial.recovery_posture()
                                    == primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable
                        ),
                        "{}: a stop before settlement offers a retry: {stop:?}",
                        $what
                    );
                    stops.push(format!("{stop:?}"));
                    None
                }
            })
            .unwrap_or_else(|| panic!("{} settles within eight advances: {stops:?}", $what))
    }};
}

#[test]
fn publication_driven_retryable_progress_is_followed_by_stable_terminal_stops() {
    let _guard = checkpoint_recovery_test_guard();
    support::capacity_region::search(
        "publication terminal",
        1,
        128 * 1024,
        support::capacity_region::Goal::Hit,
        publication_terminal,
    )
    .require_hit("publication terminal");
}

fn publication_terminal(budget: usize) -> Attempt {
    let (application, _invalidation) = limited_application(budget, 128 * 1_024 * 1_024, 8);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, mut c) = setup_chain!(application, request);
    let settled_custody = application.required_custody_bytes_for_test();

    change_root_input!(request, application, 2, 0x9176_4080_u64);
    let source_commit = request.retain_read().unwrap().selected_commit().clone();
    drop((a, b));
    take_decisions("anchor-b");

    // A's first publication fits beside the predecessor chain. Another
    // continuation slot cannot fit until the refused advance releases custody.
    // The publication justifies a retry; later unproductive stops are terminal.
    let stopped = c.advance(&request);
    if stopped.is_ok() {
        return Attempt::Above("last consumer refresh admitted");
    }
    if request.retain_read().unwrap().selected_commit() == &source_commit {
        return Attempt::Below("refused before first upstream publication");
    }
    assert!(
        matches!(
            &stopped,
            Err(WorthQueryApplicationOutputDemandDenial::Demand(denial))
                if denial.kind() == WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
                    && denial.recovery_posture()
                        == primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable
        ),
        "the partially published refresh offers a retry: {:?}",
        stopped.as_ref().err()
    );
    assert_ne!(
        request.retain_read().unwrap().selected_commit().clone(),
        source_commit,
        "the stopped advance actually published an upstream output"
    );
    let root = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        root.rows()[0].value,
        length(3),
        "the published root uses the changed input"
    );
    let stopped_custody = application.required_custody_bytes_for_test();
    assert!(
        stopped_custody <= budget,
        "partial publication retains {stopped_custody} bytes within its declared {}-byte allowance; the earlier settled baseline was {settled_custody}",
        budget
    );
    let published_commit = request.retain_read().unwrap().selected_commit().clone();
    let mut held_custody = stopped_custody;
    for attempt in 0..2 {
        let entries = query_entries();
        let stopped = c.advance(&request);
        assert!(
            matches!(
                &stopped,
                Err(WorthQueryApplicationOutputDemandDenial::Demand(denial))
                    if denial.kind() == WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
                        && denial.recovery_posture()
                            == primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal
            ),
            "attempt {attempt}: no further progress permits a retry: {:?}",
            stopped.as_ref().err()
        );
        assert!(
            query_entries() > entries,
            "the terminal attempt re-enters the real source query"
        );
        assert_eq!(
            request.retain_read().unwrap().selected_commit(),
            &published_commit,
            "a terminal attempt publishes no further output"
        );
        let after = application.required_custody_bytes_for_test();
        assert!(
            after <= held_custody,
            "terminal attempt custody grows from {held_custody} to {after}"
        );
        held_custody = after;
    }
    assert!(
        take_decisions("anchor-b").is_empty(),
        "the budget-refused middle produces no completed decision"
    );
    drop(c);
    assert!(
        application.required_custody_bytes_for_test() <= held_custody,
        "closing the stopped caller retains no excess custody"
    );
    Attempt::Hit
}

/// Overwrites the Length `anchor-b` publishes, as a writer that is not its
/// producer.
mod reclamation;

mod balance;

mod stable_readmission;

mod joined_refresh;
