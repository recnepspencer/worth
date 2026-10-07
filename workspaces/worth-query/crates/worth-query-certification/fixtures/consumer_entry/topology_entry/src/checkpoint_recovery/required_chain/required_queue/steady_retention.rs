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

/// One advance settles the unrelated demand, whose output no cycle touches.
/// Demanded every cycle, it never leaves the retained window: the advance
/// runs the source queries the dirty chain needs and none for its own
/// output. A full verification each time the window rotates would add one
/// to a cycle in every rotation, so every cycle compared runs exactly as
/// many as the first of them.
macro_rules! unrelated_settles_unverified {
    ($queries:expr, $cycle:expr, $d:expr, $request:expr) => {{
        let before = query_entries();
        settled_in_one_advance!($d, $request, "the unrelated required demand");
        let ran = query_entries() - before;
        assert_eq!(
            *$queries.get_or_insert(ran),
            ran,
            "cycle {}: the untouched output is never verified in full",
            $cycle
        );
    }};
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
        let mut queries = None;
        for cycle in 0..100_u64 {
            change_root_input!(request, application, 2 + cycle % 2, 0x9176_3d00_u64 + cycle);
            unrelated_settles_unverified!(queries, cycle, d, request);
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
    let retained_positions = 8;
    let (application, invalidation) =
        limited_application(4 * 1_024 * 1_024, 10 * 1_024 * 1_024, retained_positions);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    let mut queries = None;
    for cycle in 0..cycles {
        change_root_input!(request, application, 2 + cycle % 2, 0x9176_3c00_u64 + cycle);
        // This small index keeps every row of the chain: no registration
        // is refused, and each cycle refreshes each row once, from the first.
        unrelated_settles_unverified!(queries, cycle, d, request);
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
            application
                .history_retained_for_test()
                .expect("the retention fixture requires an open application owner"),
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

/// What the index retains once every retained version holds the same rows.
/// The chain cycles, and a field no source of the chain reads is then written
/// until every position of the window holds such a write: no edit separates
/// one retained version from the next.
fn retained_by_versions_no_edit_separates(retained_positions: usize) -> u64 {
    let (application, invalidation) =
        limited_application(4 * 1_024 * 1_024, 10 * 1_024 * 1_024, retained_positions);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    for cycle in 0..4_u64 {
        change_root_input!(request, application, 2 + cycle % 2, 0x9176_3f00_u64 + cycle);
        settled_in_one_advance!(d, request, "the unrelated required demand");
        settled_in_one_advance!(c, request, "the last consumer");
        settled_in_one_advance!(b, request, "the middle consumer");
        settled_in_one_advance!(a, request, "the open root demand");
    }
    let mut unread_write = |position: u64| {
        assert!(
            writes_y!(
                request,
                application,
                "anchor-c",
                20 + position % 2,
                0x9176_3f80_u64 + position
            ),
            "{retained_positions} retained positions: the index has room for a write no row reads"
        );
        invalidation.retained_capacity_bytes()
    };
    let window = retained_positions as u64;
    let retained = (0..=window).map(&mut unread_write).last().unwrap();
    assert_eq!(
        unread_write(window + 1),
        retained,
        "{retained_positions} retained positions: a window of writes no row reads is steady"
    );
    drop((a, b, c, d));
    retained
}

#[test]
fn retained_versions_no_edit_separates_share_one_index() {
    let _guard = checkpoint_recovery_test_guard();
    // One world retains a single version of the index and the other eight,
    // over the same rows. Versions that share hold one whole index between
    // them, so the seven further positions add only their own bookkeeping:
    // less than a second single version. Versions that shared nothing would
    // each hold the whole index.
    let single = retained_by_versions_no_edit_separates(1);
    let eight = retained_by_versions_no_edit_separates(8);
    assert!(
        single <= eight && eight - single < single,
        "eight versions over the same rows retain {eight} bytes and a single version {single}"
    );
}
