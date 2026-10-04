//! Query-owned retained bytes and required custody across many cycles.

use super::*;

/// Retained invalidation bytes and required custody, compared across cycles
/// once every retained position has rotated: from then on each cycle retains
/// exactly what the one before it did.
macro_rules! assert_steady {
    ($steady:expr, $cycle:expr, $retained_positions:expr, $application:expr, $invalidation:expr) => {
        if $cycle >= 2 * $retained_positions as u64 {
            let retained = (
                $invalidation.retained_capacity_bytes(),
                $application.required_custody_bytes_for_test(),
            );
            assert_eq!(
                *$steady.get_or_insert(retained),
                retained,
                "cycle {}: retained invalidation bytes and required custody are steady",
                $cycle
            );
        }
    };
}

#[test]
fn an_unrelated_caller_advances_for_many_cycles_under_tight_custody() {
    let _guard = checkpoint_recovery_test_guard();
    let row = worth_query_host::facade::primary_graph::required_ready_custody_bytes_for_test();
    // Three Ready rows per settled demand fail every queued refresh the
    // unrelated caller runs as a frame; five let those refreshes finish. A
    // refresh that cannot finish returns its row to the Ready it reopened,
    // which stays cached for an equivalent later demand like any Ready row.
    for rows_per_demand in [3, 5] {
        let retained_positions = 8;
        let (application, invalidation) = limited_application(
            4 * rows_per_demand * row,
            128 * 1_024 * 1_024,
            retained_positions,
        );
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let (a, b, c, mut d) = chain_with_unrelated!(application, request);
        let settled_custody = application.required_custody_bytes_for_test();
        let mut steady = None;
        for cycle in 0..100_u64 {
            change_root_input!(request, application, 2 + cycle % 2, 0x9176_3d00_u64 + cycle);
            settled_in_one_advance!(d, request, "the unrelated required demand");
            assert_steady!(steady, cycle, retained_positions, application, invalidation);
        }
        // A refreshed row belongs to its own demand, never to the caller
        // that happened to run it, so closing the chain leaves no more custody
        // than the settled chain held. The middle consumer then reopens alone
        // over its stale, undemanded root. At five rows it refreshes that
        // root and settles in one advance. At three, custody cannot keep the
        // middle refresh at once: the advance stops retryable and ends the
        // refreshes it carried, which leaves less custody held than before
        // it, so the retry settles beside the unrelated caller's open Ready.
        drop((a, b, c));
        let closed_custody = application.required_custody_bytes_for_test();
        assert!(
            closed_custody <= settled_custody,
            "{rows_per_demand} rows per demand: closing the chain holds no more than it settled with"
        );
        let mut b = request
            .demand(ChainDemand("anchor-b".to_owned()))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                &application,
            )
            .unwrap();
        if rows_per_demand == 3 {
            let stopped = b.advance(&request);
            assert!(
                matches!(
                    &stopped,
                    Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
                        if denial.kind() == WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
                            && denial.recovery_posture()
                                == primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable
                ),
                "a refresh custody cannot keep at once stops retryable: {:?}",
                stopped.as_ref().err()
            );
            assert!(
                application.required_custody_bytes_for_test() < closed_custody,
                "the stop holds less than the advance began with"
            );
        }
        settled_in_one_advance!(b, request, "the reopened middle consumer");
        drop(b);
        assert!(
            application.required_custody_bytes_for_test() <= settled_custody,
            "{rows_per_demand} rows per demand: the unrelated caller retains no refreshed chain row"
        );
        settled_in_one_advance!(d, request, "the unrelated required demand");
        drop(d);
        assert!(
            application.required_custody_bytes_for_test() <= settled_custody,
            "{rows_per_demand} rows per demand: custody returns within the settled chain's"
        );
    }
}

/// Query-owned state one cycle leaves retained: invalidation index bytes,
/// required custody, output-lineage history, and the completed evidence
/// entries inside the idempotency window with the bytes their tickets hold.
type QueryRetained = (u64, usize, u64, usize, usize);

/// History one cycle leaves retained: installed World commits, World history
/// metadata bytes, unique World component pins and Relational retired roots.
type HistoryRetained = (usize, usize, usize, usize);

/// Runs the full chain at small retention for `cycles` cycles, each settling
/// every demand in one advance, and reports what each cycle leaves retained.
fn cycle_chain_at_small_retention(
    cycles: u64,
    mut retained: impl FnMut(u64, QueryRetained, HistoryRetained),
) {
    let (application, invalidation) = limited_application(4 * 1_024 * 1_024, 8 * 1_024 * 1_024, 8);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    for cycle in 0..cycles {
        change_root_input!(request, application, 2 + cycle % 2, 0x9176_3c00_u64 + cycle);
        settled_in_one_advance!(d, request, "the unrelated required demand");
        settled_in_one_advance!(c, request, "the last consumer");
        settled_in_one_advance!(b, request, "the middle consumer");
        settled_in_one_advance!(a, request, "the open root demand");
        let (evidence_entries, evidence_bytes) = application.completed_evidence_retained_for_test();
        retained(
            cycle,
            (
                invalidation.retained_capacity_bytes(),
                application.required_custody_bytes_for_test(),
                application.output_lineage_retained_bytes_for_test(),
                evidence_entries,
                evidence_bytes,
            ),
            application.history_retained_for_test(),
        );
    }
    drop((a, b, c, d));
}

#[test]
fn a_refreshed_row_releases_the_custody_of_the_one_it_supersedes() {
    let _guard = checkpoint_recovery_test_guard();
    // A demand refreshing its own row again mints a successor of that same
    // row; the one before it is superseded, so custody stops growing once
    // both alternating inputs have refreshed the chain.
    let mut steady = None;
    cycle_chain_at_small_retention(12, |cycle, (_, custody, ..), _| {
        if cycle >= 2 {
            assert_eq!(
                *steady.get_or_insert(custody),
                custody,
                "cycle {cycle}: required custody is steady"
            );
        }
    });
}

#[test]
fn the_required_chain_stays_live_for_a_hundred_cycles_at_small_retention() {
    let _guard = checkpoint_recovery_test_guard();
    let cycles = 100;
    // World capacity is the fixture's own: history behind the settled head
    // retires each cycle, and the idempotency window evicts the oldest
    // completed evidence once full, so World, Relational and Query retention
    // all stop growing once every one of the eight retained positions has
    // rotated.
    let mut steady = None;
    let mut steady_history = None;
    cycle_chain_at_small_retention(cycles, |cycle, retained, history| {
        if cycle >= 16 {
            assert_eq!(
                *steady.get_or_insert(retained),
                retained,
                "cycle {cycle}: Query-owned retained bytes are steady"
            );
            assert_eq!(
                *steady_history.get_or_insert(history),
                history,
                "cycle {cycle}: World history, pins and Relational retired roots are steady"
            );
        }
    });
}
