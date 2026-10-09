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
    // Both finite profiles admit the four-demand world. Twelve actual Ready
    // layout slots admit the long-cycle reopen; twenty provide the
    // contrasting headroom. Retry/terminal pressure is covered separately
    // in custody_stops rather than assumed from a stale byte estimate here.
    for custody_rows in [12, 20] {
        let retained_positions = 8;
        let custody_budget = custody_rows * row;
        let (application, invalidation) =
            limited_application(custody_budget, 128 * 1_024 * 1_024, retained_positions);
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
        // that happened to run it. The middle consumer then reopens alone
        // over its stale, undemanded root beside the existing unrelated caller.
        drop((a, b, c));
        let closed_custody = application.required_custody_bytes_for_test();
        assert!(
            closed_custody <= settled_custody,
            "{custody_rows} Ready slots: closing the chain holds no more than it settled with"
        );
        let mut b = request
            .demand(ChainDemand("anchor-b".to_owned()))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                &application,
            )
            .unwrap();
        settled_in_one_advance!(b, request, "the reopened middle consumer");
        drop(b);
        let closed_ready_custody = application.required_custody_bytes_for_test();
        assert!(
            closed_ready_custody <= custody_budget,
            "{custody_rows} Ready slots: eligible closed Ready custody remains in its declared allowance"
        );
        // Closing a caller may leave its Ready cached. Reopening unchanged
        // output must neither accumulate those eligible rows nor republish it.
        let current_commit = request.retain_read().unwrap().selected_commit().clone();
        for reopen in 0..8 {
            let mut b = request
                .demand(ChainDemand("anchor-b".to_owned()))
                .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                    &application,
                )
                .unwrap();
            settled_in_one_advance!(b, request, "the unchanged reopened middle consumer");
            drop(b);
            assert_eq!(
                application.required_custody_bytes_for_test(),
                closed_ready_custody,
                "reopen {reopen}: closed Ready custody stays on its actual plateau"
            );
            assert_eq!(
                request.retain_read().unwrap().selected_commit(),
                &current_commit,
                "reopen {reopen}: unchanged output is not republished"
            );
        }
        settled_in_one_advance!(d, request, "the unrelated required demand");
        drop(d);
        assert!(
            application.required_custody_bytes_for_test() <= closed_ready_custody,
            "{custody_rows} Ready slots: closing the unrelated caller adds no custody"
        );
    }
}

/// Query-owned state one cycle leaves retained: invalidation index bytes,
/// required custody, output-lineage history, and the completed evidence
/// entries that retain the exact committed idempotency evidence.
type QueryRetained = (u64, usize, u64, usize);

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
        let evidence_entries = application.completed_evidence_retained_for_test();
        retained(
            cycle,
            (
                invalidation.retained_capacity_bytes(),
                application.required_custody_bytes_for_test(),
                application.output_lineage_retained_bytes_for_test(),
                evidence_entries,
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
    // retires each cycle. Invalidation, required custody and lineage rotate;
    // committed idempotency evidence remains retained independently.
    let mut steady = None;
    let mut steady_history = None;
    let mut prior_evidence = 0;
    cycle_chain_at_small_retention(
        cycles,
        |cycle, (invalidation, custody, lineage, evidence), history| {
            assert!(
                evidence >= prior_evidence,
                "completed idempotency evidence is retained"
            );
            prior_evidence = evidence;
            let retained = (invalidation, custody, lineage);
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
        },
    );
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
