//! Journey 5: residual evaluation is metered by the request and stops at
//! its safe points with typed denials.
use super::world::{funded_search, Host, Seed};
use super::*;
use std::cell::Cell;
use std::rc::Rc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::read_execution::interruption::checkpoint_observer::observe_checkpoints_for_test;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial as Denial;
use worth_query_host::facade::primary_graph::WorthQueryApplicationOneShotDenialKind as Kind;

/// Twenty-four project-1 tasks whose filter-only notes are `width` bytes.
fn quiet_board(width: usize) -> Vec<Seed> {
    (1..=24)
        .map(|key| {
            (
                key,
                1,
                "task",
                Some("n".repeat(width)),
                "open",
                1,
                1_000_000,
                None,
            )
        })
        .collect()
}

fn cancellable() -> (WorthQueryCancellationSource, WorthQueryRequestScope) {
    let source = WorthQueryCancellationSource::new();
    let scope =
        WorthQueryRequestScope::new(None, source.token(), WorthQueryAdmissionClock::system());
    (source, scope)
}

/// The smallest caller budget that completes a search matching no row, so
/// residual evaluation is the last metered work of the read.
fn minimal_budget(host: &Host) -> u64 {
    let (mut low, mut high) = (1_u64, 1 << 24);
    let (_, scope) = cancellable();
    assert!(funded_search(host, &scope, high, search_text(1, "absent")).is_ok());
    while low < high {
        let middle = low + (high - low) / 2;
        match funded_search(host, &scope, middle, search_text(1, "absent")) {
            Ok(_) => high = middle,
            Err(_) => low = middle + 1,
        }
    }
    low
}

#[test]
fn filter_evaluation_is_charged_to_the_request_and_exhaustion_refuses_typed() {
    let short = install(&quiet_board(8));
    let long = install(&quiet_board(4096));
    let (short_budget, long_budget) = (minimal_budget(&short), minimal_budget(&long));
    // Notes are never projected: only residual evaluation scans them.
    assert!(
        long_budget > short_budget + 24 * 256,
        "scanning 24 × 4 KiB of notes must cost the request: {short_budget} vs {long_budget}"
    );
    let (_, scope) = cancellable();
    let (rows, read_work) = funded_search(&long, &scope, long_budget, search_text(1, "absent"))
        .expect("the minimal budget completes");
    assert!(rows.is_empty());
    assert!(
        read_work > 24 * 256,
        "the receipt reports filter work: {read_work}"
    );
    match funded_search(&long, &scope, long_budget - 1, search_text(1, "absent")) {
        Err(Denial::Execution(denial)) => {
            assert!(
                matches!(denial.kind(), Kind::ExecutionRequest(_)),
                "{denial:?}"
            );
            assert_eq!(denial.subject(), "where_filter");
        }
        other => panic!("one unit short must refuse inside the filter: {other:?}"),
    }
}

#[test]
fn cancellation_between_residual_slices_refuses_typed_after_partial_evaluation() {
    let host = install(&quiet_board(64));
    let (source, scope) = cancellable();
    let visits = Rc::new(Cell::new(0_usize));
    let seen = Rc::clone(&visits);
    let _observer = observe_checkpoints_for_test(Box::new(move |subject| {
        if subject == "where_filter" {
            seen.set(seen.get() + 1);
            if seen.get() == 5 {
                source.cancel();
            }
        }
    }));
    match funded_search(&host, &scope, 1 << 24, search_text(1, "absent")) {
        Err(Denial::Execution(denial)) => assert_eq!(denial.kind(), Kind::Cancelled),
        other => panic!("a cancelled request must stop inside the filter: {other:?}"),
    }
    assert_eq!(
        visits.get(),
        5,
        "evaluation stops at the first safe point after cancel"
    );
}
