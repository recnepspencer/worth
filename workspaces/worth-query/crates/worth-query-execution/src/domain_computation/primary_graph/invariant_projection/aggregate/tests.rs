//! Real-reader hostile proofs for aggregate execution, accounting, and cache reuse.

mod world;

use super::WorthQueryInvariantAggregateDenialKind;
use world::AggregateWorld;

#[test]
fn cold_warm_and_stale_snapshots_follow_real_coordinator_phases() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = AggregateWorld::values([7]);
    world
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let cold = world.observe(&(world.fixture_phase(&host_custody)).test_host_request());
            assert_eq!(cold.result, Ok((7, 1)));
            assert_eq!(cold.work.aggregate_lookups(), 1);
            assert_eq!(cold.work.aggregate_cache_hits(), 0);
            assert_eq!(cold.work.aggregate_rebuild_input_rows(), 1);

            let warm = world.observe(&(world.fixture_phase(&host_custody)).test_host_request());
            assert_eq!(warm.result, Ok((7, 1)));
            assert_eq!(warm.work.aggregate_lookups(), 1);
            assert_eq!(warm.work.aggregate_cache_hits(), 1);
            assert_eq!(warm.work.aggregate_rebuild_input_rows(), 0);

            world.replace_amount(
                execution,
                &world.fixture_phase(&host_custody),
                "source-0",
                11,
            );
            let stale = world.observe(&(world.fixture_phase(&host_custody)).test_host_request());
            assert_eq!(stale.result, Ok((11, 1)));
            assert_eq!(stale.work.aggregate_lookups(), 1);
            assert_eq!(stale.work.aggregate_cache_hits(), 0);
            assert_eq!(stale.work.aggregate_rebuild_input_rows(), 1);
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn prelookup_and_midscan_budgets_deny_on_the_real_relational_reads() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let world = AggregateWorld::values([3, 4]);
    let before_lookup = world.observe_bounded(execution, 0);
    assert!(before_lookup.exhausted);
    assert_eq!(
        before_lookup.denial,
        Some(WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded)
    );
    assert_eq!(before_lookup.work.aggregate_lookups(), 0);
    assert_eq!(before_lookup.work.adjacency_edges_inspected(), 0);
    let midscan = world.observe_bounded(execution, 3);
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
    assert_eq!(midscan.work.endpoint_records_read(), 0);
    assert_eq!(midscan.work.field_reads(), 0);
    world
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let retry = world.observe(&(world.fixture_phase(&host_custody)).test_host_request());
            assert_eq!(retry.result, Ok((7, 2)));
            assert_eq!(retry.work.aggregate_cache_hits(), 0);
            assert_eq!(retry.work.aggregate_rebuild_input_rows(), 2);
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn outgoing_and_field_budgets_count_only_performed_work_and_do_not_poison() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let outgoing_denial = AggregateWorld::values([7]).observe_bounded(execution, 3);
    assert!(outgoing_denial.exhausted);
    assert_eq!(
        outgoing_denial.denial,
        Some(WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded)
    );
    assert_eq!(outgoing_denial.work.aggregate_lookups(), 1);
    assert_eq!(outgoing_denial.work.aggregate_rebuild_input_rows(), 1);
    assert_eq!(outgoing_denial.work.adjacency_lists_read(), 2);
    assert_eq!(outgoing_denial.work.adjacency_edges_inspected(), 1);
    assert_eq!(outgoing_denial.work.endpoint_records_read(), 0);
    assert_eq!(outgoing_denial.work.field_reads(), 0);
    let field_world = AggregateWorld::values([7]);
    let field_denial = field_world.observe_bounded(execution, 5);
    assert!(field_denial.exhausted);
    assert_eq!(
        field_denial.denial,
        Some(WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded)
    );
    assert_eq!(field_denial.work.aggregate_lookups(), 1);
    assert_eq!(field_denial.work.aggregate_rebuild_input_rows(), 1);
    assert_eq!(field_denial.work.adjacency_lists_read(), 2);
    assert_eq!(field_denial.work.adjacency_edges_inspected(), 2);
    assert_eq!(field_denial.work.endpoint_records_read(), 0);
    assert_eq!(field_denial.work.field_reads(), 0);
    field_world
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let field_retry = field_world.observe(&host_custody);
            assert_eq!(field_retry.result, Ok((7, 1)));
            assert_eq!(field_retry.work.aggregate_cache_hits(), 0);
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn partial_outgoing_denial_carries_exact_scan_work_and_does_not_poison() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let ambiguous_world = AggregateWorld::ambiguous(7);
    let partial_outgoing = ambiguous_world.observe_bounded(execution, 6);
    assert!(partial_outgoing.exhausted);
    assert_eq!(
        partial_outgoing.denial,
        Some(WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded)
    );
    assert_eq!(partial_outgoing.work.aggregate_lookups(), 1);
    assert_eq!(partial_outgoing.work.aggregate_rebuild_input_rows(), 1);
    assert_eq!(partial_outgoing.work.adjacency_lists_read(), 2);
    assert_eq!(partial_outgoing.work.adjacency_edges_inspected(), 3);
    assert_eq!(partial_outgoing.work.endpoint_records_read(), 0);
    assert_eq!(partial_outgoing.work.field_reads(), 0);
    ambiguous_world
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let ambiguous_retry = ambiguous_world.observe(&host_custody);
            assert_eq!(
                ambiguous_retry.result,
                Err(WorthQueryInvariantAggregateDenialKind::AmbiguousSourceRelation)
            );
            assert_eq!(ambiguous_retry.work.aggregate_cache_hits(), 0);
            assert_eq!(ambiguous_retry.work.field_reads(), 0);
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn ambiguous_source_denial_records_work_and_never_poisons_the_cache() {
    let world = AggregateWorld::ambiguous(7);
    world
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            assert_repeatable_denial(
                &world.fixture_phase(&host_custody),
                &world,
                DenialWorkExpectation {
                    kind: WorthQueryInvariantAggregateDenialKind::AmbiguousSourceRelation,
                    rows: 1,
                    adjacency_edges: 3,
                    field_reads: 0,
                },
            );
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn absent_scalar_denial_records_work_and_never_poisons_the_cache() {
    let world = AggregateWorld::missing_value();
    world
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            assert_repeatable_denial(
                &world.fixture_phase(&host_custody),
                &world,
                DenialWorkExpectation {
                    kind: WorthQueryInvariantAggregateDenialKind::InvalidScalar,
                    rows: 1,
                    adjacency_edges: 2,
                    field_reads: 1,
                },
            );
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn arithmetic_overflow_records_every_examined_row_without_cache_poisoning() {
    let world = AggregateWorld::values([i64::MAX, 1]);
    world
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            assert_repeatable_denial(
                &world.fixture_phase(&host_custody),
                &world,
                DenialWorkExpectation {
                    kind: WorthQueryInvariantAggregateDenialKind::ArithmeticOverflow,
                    rows: 2,
                    adjacency_edges: 4,
                    field_reads: 2,
                },
            );
        })
        .expect("the fixture block owns its installed host call")
}

struct DenialWorkExpectation {
    kind: WorthQueryInvariantAggregateDenialKind,
    rows: usize,
    adjacency_edges: usize,
    field_reads: usize,
}

fn assert_repeatable_denial(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AggregateWorld,
    expected: DenialWorkExpectation,
) {
    for attempt in 0..2 {
        let observed = world.observe(&(phase_custody).test_host_request());
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
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = AggregateWorld::values([3, 4]);
    let main = world.main_branch();
    world
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let (first, second) = (
                world.source(&world.fixture_phase(&host_custody), "source-0"),
                world.source(&world.fixture_phase(&host_custody), "source-1"),
            );
            let removed = world.contribution(&world.fixture_phase(&host_custody), second);
            let fork = world.fork(&world.fixture_phase(&host_custody), main);
            // The parent drops a contribution after the fork, then the fork commits
            // last, so the fork reads at a version newer than the parent's removal.
            world.remove_relation_on(
                execution,
                &world.fixture_phase(&host_custody),
                main,
                removed,
            );
            world.replace_amount_on(
                execution,
                &world.fixture_phase(&host_custody),
                fork,
                first,
                5,
            );

            assert_eq!(
                world.observe_on(&world.fixture_phase(&host_custody), fork),
                Ok((9, 2))
            );
            assert_eq!(
                world.observe_on(&world.fixture_phase(&host_custody), main),
                Ok((3, 1))
            );
        })
        .expect("the fixture block owns its installed host call")
}
