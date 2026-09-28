//! Real-reader hostile proofs for aggregate execution, accounting, and cache reuse.

mod world;

use super::WorthQueryInvariantAggregateDenialKind;
use world::AggregateWorld;

#[test]
fn cold_warm_and_stale_snapshots_follow_real_coordinator_phases() {
    let world = AggregateWorld::values([7]);

    let cold = world.observe();
    assert_eq!(cold.result, Ok((7, 1)));
    assert_eq!(cold.work.aggregate_lookups(), 1);
    assert_eq!(cold.work.aggregate_cache_hits(), 0);
    assert_eq!(cold.work.aggregate_rebuild_input_rows(), 1);

    let warm = world.observe();
    assert_eq!(warm.result, Ok((7, 1)));
    assert_eq!(warm.work.aggregate_lookups(), 1);
    assert_eq!(warm.work.aggregate_cache_hits(), 1);
    assert_eq!(warm.work.aggregate_rebuild_input_rows(), 0);

    world.replace_amount("source-0", 11);
    let stale = world.observe();
    assert_eq!(stale.result, Ok((11, 1)));
    assert_eq!(stale.work.aggregate_lookups(), 1);
    assert_eq!(stale.work.aggregate_cache_hits(), 0);
    assert_eq!(stale.work.aggregate_rebuild_input_rows(), 1);
}

#[test]
fn prelookup_and_midscan_budgets_deny_on_the_real_relational_reads() {
    let world = AggregateWorld::values([3, 4]);

    let before_lookup = world.observe_bounded(0);
    assert!(before_lookup.exhausted);
    assert_eq!(
        before_lookup.denial,
        Some(WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded)
    );
    assert_eq!(before_lookup.work.aggregate_lookups(), 0);
    assert_eq!(before_lookup.work.adjacency_edges_inspected(), 0);

    let midscan = world.observe_bounded(3);
    assert!(midscan.exhausted);
    assert_eq!(
        midscan.denial,
        Some(WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded)
    );
    assert_eq!(midscan.work.aggregate_lookups(), 1);
    assert_eq!(midscan.work.aggregate_cache_hits(), 0);
    assert_eq!(midscan.work.aggregate_rebuild_input_rows(), 1);
    assert_eq!(midscan.work.adjacency_lists_read(), 1);
    assert_eq!(midscan.work.adjacency_edges_inspected(), 1);
    assert_eq!(midscan.work.endpoint_records_read(), 1);
    assert_eq!(midscan.work.field_reads(), 0);

    let retry = world.observe();
    assert_eq!(retry.result, Ok((7, 2)));
    assert_eq!(retry.work.aggregate_cache_hits(), 0);
    assert_eq!(retry.work.aggregate_rebuild_input_rows(), 2);
}

#[test]
fn outgoing_and_field_budgets_count_only_performed_work_and_do_not_poison() {
    let outgoing_denial = AggregateWorld::values([7]).observe_bounded(3);
    assert!(outgoing_denial.exhausted);
    assert_eq!(
        outgoing_denial.denial,
        Some(WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded)
    );
    assert_eq!(outgoing_denial.work.aggregate_lookups(), 1);
    assert_eq!(outgoing_denial.work.aggregate_rebuild_input_rows(), 1);
    assert_eq!(outgoing_denial.work.adjacency_lists_read(), 2);
    assert_eq!(outgoing_denial.work.adjacency_edges_inspected(), 1);
    assert_eq!(outgoing_denial.work.endpoint_records_read(), 1);
    assert_eq!(outgoing_denial.work.field_reads(), 0);

    let field_world = AggregateWorld::values([7]);
    let field_denial = field_world.observe_bounded(5);
    assert!(field_denial.exhausted);
    assert_eq!(
        field_denial.denial,
        Some(WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded)
    );
    assert_eq!(field_denial.work.aggregate_lookups(), 1);
    assert_eq!(field_denial.work.aggregate_rebuild_input_rows(), 1);
    assert_eq!(field_denial.work.adjacency_lists_read(), 2);
    assert_eq!(field_denial.work.adjacency_edges_inspected(), 2);
    assert_eq!(field_denial.work.endpoint_records_read(), 2);
    assert_eq!(field_denial.work.field_reads(), 0);
    let field_retry = field_world.observe();
    assert_eq!(field_retry.result, Ok((7, 1)));
    assert_eq!(field_retry.work.aggregate_cache_hits(), 0);
}

#[test]
fn partial_outgoing_denial_carries_exact_scan_work_and_does_not_poison() {
    let ambiguous_world = AggregateWorld::ambiguous(7);
    let partial_outgoing = ambiguous_world.observe_bounded(6);
    assert!(partial_outgoing.exhausted);
    assert_eq!(
        partial_outgoing.denial,
        Some(WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded)
    );
    assert_eq!(partial_outgoing.work.aggregate_lookups(), 1);
    assert_eq!(partial_outgoing.work.aggregate_rebuild_input_rows(), 1);
    assert_eq!(partial_outgoing.work.adjacency_lists_read(), 2);
    assert_eq!(partial_outgoing.work.adjacency_edges_inspected(), 3);
    assert_eq!(partial_outgoing.work.endpoint_records_read(), 2);
    assert_eq!(partial_outgoing.work.field_reads(), 0);
    let ambiguous_retry = ambiguous_world.observe();
    assert_eq!(
        ambiguous_retry.result,
        Err(WorthQueryInvariantAggregateDenialKind::AmbiguousSourceRelation)
    );
    assert_eq!(ambiguous_retry.work.aggregate_cache_hits(), 0);
    assert_eq!(ambiguous_retry.work.field_reads(), 0);
}

#[test]
fn ambiguous_source_denial_records_work_and_never_poisons_the_cache() {
    let world = AggregateWorld::ambiguous(7);
    assert_repeatable_denial(
        &world,
        DenialWorkExpectation {
            kind: WorthQueryInvariantAggregateDenialKind::AmbiguousSourceRelation,
            rows: 1,
            adjacency_edges: 3,
            field_reads: 0,
        },
    );
}

#[test]
fn absent_scalar_denial_records_work_and_never_poisons_the_cache() {
    let world = AggregateWorld::missing_value();
    assert_repeatable_denial(
        &world,
        DenialWorkExpectation {
            kind: WorthQueryInvariantAggregateDenialKind::InvalidScalar,
            rows: 1,
            adjacency_edges: 2,
            field_reads: 1,
        },
    );
}

#[test]
fn arithmetic_overflow_records_every_examined_row_without_cache_poisoning() {
    let world = AggregateWorld::values([i64::MAX, 1]);
    assert_repeatable_denial(
        &world,
        DenialWorkExpectation {
            kind: WorthQueryInvariantAggregateDenialKind::ArithmeticOverflow,
            rows: 2,
            adjacency_edges: 4,
            field_reads: 2,
        },
    );
}

struct DenialWorkExpectation {
    kind: WorthQueryInvariantAggregateDenialKind,
    rows: usize,
    adjacency_edges: usize,
    field_reads: usize,
}

fn assert_repeatable_denial(world: &AggregateWorld, expected: DenialWorkExpectation) {
    for attempt in 0..2 {
        let observed = world.observe();
        assert_eq!(
            observed.result,
            Err(expected.kind),
            "denial attempt {attempt}"
        );
        assert_eq!(observed.work.aggregate_lookups(), 1);
        assert_eq!(observed.work.aggregate_cache_hits(), 0);
        assert_eq!(observed.work.aggregate_rebuild_input_rows(), expected.rows);
        assert_eq!(
            observed.work.adjacency_edges_inspected(),
            expected.adjacency_edges
        );
        assert_eq!(observed.work.field_reads(), expected.field_reads);
    }
}

#[test]
fn a_fork_summarizes_only_its_own_contributions_after_its_parent_moves_on() {
    let world = AggregateWorld::values([3, 4]);
    let main = world.main_branch();
    let (first, second) = (world.source("source-0"), world.source("source-1"));
    let removed = world.contribution(second);
    let fork = world.fork(main);
    // The parent drops a contribution after the fork, then the fork commits
    // last, so the fork reads at a version newer than the parent's removal.
    world.remove_relation_on(main, removed);
    world.replace_amount_on(fork, first, 5);

    assert_eq!(world.observe_on(fork), Ok((9, 2)));
    assert_eq!(world.observe_on(main), Ok((3, 1)));
}
