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
    // unrelated caller runs as a frame; five let those refreshes finish.
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
        // Product World keeps every live-branch commit, so the cycle count
        // stays within the fixture's World retention.
        for cycle in 0..40_u64 {
            change_root_input!(request, application, 2 + cycle % 2, 0x9176_3d00_u64 + cycle);
            settled_in_one_advance!(d, request, "the unrelated required demand");
            assert_steady!(steady, cycle, retained_positions, application, invalidation);
        }
        // A refreshed row belongs to its own demand, never to the caller
        // that happened to run it: once the chain's demands close, custody
        // is back within what the settled set held.
        drop((a, b, c));
        let closed_custody = application.required_custody_bytes_for_test();
        assert!(
            closed_custody <= settled_custody,
            "{rows_per_demand} rows per demand: the unrelated caller retains no refreshed chain row"
        );
        settled_in_one_advance!(d, request, "the unrelated required demand");
        drop(d);
    }
}

/// Query-owned bytes one cycle leaves retained: the invalidation index,
/// required custody and output-lineage history.
type QueryRetained = (u64, usize, u64);

/// Runs the full chain at small retention for `cycles` cycles, each settling
/// every demand in one advance, and reports what each cycle leaves retained.
fn cycle_chain_at_small_retention(
    cycles: u64,
    world_journeys: u64,
    mut retained: impl FnMut(u64, QueryRetained),
) {
    let (application, invalidation) =
        limited_application_for_journeys(4 * 1_024 * 1_024, 8 * 1_024 * 1_024, 8, world_journeys);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    for cycle in 0..cycles {
        change_root_input!(request, application, 2 + cycle % 2, 0x9176_3c00_u64 + cycle);
        settled_in_one_advance!(d, request, "the unrelated required demand");
        settled_in_one_advance!(c, request, "the last consumer");
        settled_in_one_advance!(b, request, "the middle consumer");
        settled_in_one_advance!(a, request, "the open root demand");
        retained(
            cycle,
            (
                invalidation.retained_capacity_bytes(),
                application.required_custody_bytes_for_test(),
                application.output_lineage_retained_bytes_for_test(),
            ),
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
    cycle_chain_at_small_retention(12, 1, |cycle, (_, custody, _)| {
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
    // Product World keeps every live-branch commit, with its history and pins
    // (deferred: "Live-branch history reclamation for Product World"). One
    // cycle commits the input change and one refreshed output per demand,
    // fewer commits than the twelve-cycle journey above runs within the
    // fixture's own World capacity, so World gets that capacity per cycle.
    let world_journeys = cycles;
    let mut steady = None;
    cycle_chain_at_small_retention(cycles, world_journeys, |cycle, retained| {
        // Every one of the eight retained positions has rotated.
        if cycle >= 16 {
            assert_eq!(
                *steady.get_or_insert(retained),
                retained,
                "cycle {cycle}: Query-owned retained bytes are steady"
            );
        }
    });
}
