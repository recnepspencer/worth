//! A caller refused required custody stops for itself, never for its rows.

use super::*;
use std::time::{Duration, Instant};
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
    ($application:expr, $request:expr) => {{
        let mut a = start_root!($application, $request);
        let mut b = start_consumer!($application, $request, "anchor-b");
        let mut c = start_consumer!($application, $request, "anchor-c");
        settle!(a, $request);
        settle!(b, $request);
        settle!(c, $request);
        (a, b, c)
    }};
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
fn a_lone_caller_whose_chain_outgrows_its_budget_stops_without_retry() {
    let _guard = checkpoint_recovery_test_guard();
    let row = worth_query_host::facade::primary_graph::required_ready_custody_bytes_for_test();
    // Eight rows keep the settled chain, and are too few for the last
    // consumer's caller to refresh all of it at once.
    let (application, _invalidation) = limited_application(8 * row, 128 * 1_024 * 1_024, 8);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, mut c) = chain!(application, request);
    let settled_custody = application.required_custody_bytes_for_test();
    change_root_input!(request, application, 2, 0x9176_4000_u64);
    // The last consumer's caller is the only open demand, so every row that
    // holds custody is one its own advance needs: no other demand's advance
    // or close frees any.
    drop((a, b));
    for attempt in 0..3 {
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
    settled_within_eight_advances!(a, request, "the root after closing the last consumer");
    let mut b = start_consumer!(application, request, "anchor-b");
    settled_within_eight_advances!(b, request, "the middle after closing the last consumer");
    drop((a, b));
}

#[test]
fn a_refresh_cancelled_under_tight_custody_settles_on_the_next_advance() {
    let _guard = checkpoint_recovery_test_guard();
    let row = worth_query_host::facade::primary_graph::required_ready_custody_bytes_for_test();
    // Thirteen rows keep the settled chain beside one refresh of it, with no
    // room for what an interrupted refresh would leave reserved.
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
    settled_in_one_advance!(b, request, "the cancelled caller");
    settled_in_one_advance!(c, request, "the last consumer");
    settled_in_one_advance!(a, request, "the open root demand");
    drop((a, b, c));
}

/// Overwrites the Length `anchor-b` publishes, as a writer that is not its
/// producer.
fn overwrite_middle_output(
    application: &application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        program::ChainProgram,
    >,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        CheckpointSchema,
    >,
    value: u64,
    idempotency: u64,
) {
    use worth_query_consumer_values::{PlanarDerivedOutput, PlanarOperation};
    let selected = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .unwrap();
    let outcome = request
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-b".to_owned(),
            operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                body_key: "anchor-b".to_owned(),
                value: length(value),
            }),
            validator_work: 4_096,
        }))
        .expect_source(selected.observed_sources()[0].clone())
        .idempotency(&idempotency)
        .execute_in_program::<program::ChainProgram>(application);
    assert!(
        matches!(
            &outcome,
            Ok(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed { .. })
        ),
        "the middle output is overwritten: {outcome:?}"
    );
}

/// A chain node republishes the Length its own source reads, so its output
/// does not vary with the root. Where `overwritten`, another writer changes
/// the middle output every cycle: the last consumer's decision then reads a
/// value no earlier cycle published.
#[test]
fn a_dependent_whose_row_was_reclaimed_decides_again_over_a_refreshed_upstream() {
    let _guard = checkpoint_recovery_test_guard();
    let row = worth_query_host::facade::primary_graph::required_ready_custody_bytes_for_test();
    // Eight to ten rows keep the settled chain and too little beside it: the
    // refresh of one row reclaims the closed cached row of another.
    for (rows, overwritten) in [8_usize, 9, 10]
        .into_iter()
        .flat_map(|rows| [(rows, false), (rows, true)])
    {
        let (application, _invalidation) = limited_application(rows * row, 128 * 1_024 * 1_024, 8);
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let (a, b, c) = chain!(application, request);
        drop((a, b, c));
        for cycle in 0..4_u64 {
            let at = format!("{rows} rows, cycle {cycle}, overwritten {overwritten}");
            let y = 2 + (cycle % 2) * 3;
            let idempotency =
                0x9176_4200_u64 + u64::from(overwritten) * 0x400 + rows as u64 * 16 + cycle;
            change_root_input!(request, application, y, idempotency);
            let mut a = start_root!(application, request);
            settled_within_eight_advances!(a, request, format!("{at}: the root"));
            drop(a);
            let middle = if overwritten {
                overwrite_middle_output(&application, &request, 40 + cycle, idempotency + 8);
                40 + cycle
            } else {
                16
            };
            take_decisions("anchor-b");
            let mut b = start_consumer!(application, request, "anchor-b");
            settled_within_eight_advances!(b, request, format!("{at}: the middle consumer"));
            drop(b);
            assert_eq!(
                take_decisions("anchor-b"),
                [[y + 1]],
                "{at}: the middle consumer decides over the changed root"
            );
            // Those refreshes reclaimed the last consumer's cached row, and
            // replaced the output its lineage consumed. Restarted, it neither
            // reuses its evicted output nor awaits the replaced one.
            take_decisions("anchor-c");
            let mut c = start_consumer!(application, request, "anchor-c");
            settled_within_eight_advances!(c, request, format!("{at}: the last consumer"));
            drop(c);
            assert_eq!(
                take_decisions("anchor-c"),
                [[middle]],
                "{at}: the last consumer decides again over the refreshed output"
            );
        }
    }
}
