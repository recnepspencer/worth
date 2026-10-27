//! Retained original bodies distinguish Native membership from physical dependencies.
use super::*;
use crate::domain_computation::primary_graph::component_selection::ComponentSelection;
use std::sync::Arc;
use worth_execution::{ExecutionAllocationPolicy as Allocation, ExecutionRequest};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub(super) fn body<Q>(
    world: &World,
    source: &WorthQueryObservedSource<Q>,
    request: &WorthQueryRequestScope,
) -> Arc<ComponentSelection> {
    let layout = world.application.runtime.primary_graph().unwrap().layout();
    let facts = source
        .retained_checkpoint_facts(layout, Some(request), Allocation::SystemAllocation)
        .unwrap();
    let mut bodies = facts.iter().filter_map(|fact| match fact {
        WorthQueryApplicationObservedFact::ComponentSelection { observation } => {
            Some(Arc::clone(observation))
        }
        _ => None,
    });
    let selected = bodies
        .next()
        .expect("component source retains its original body");
    assert!(bodies.next().is_none());
    selected
}
#[test]
fn component_rows_share_cohort_custody_and_census_retains_complete_set_truth() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = interleaved_world();
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let result = read::run(&world, Census::reference(), 0);
    let full = body(&world, result.result_set_observation().source(), &request);
    assert_eq!(full.aggregates().len(), 2);
    assert!(
        full.operation_scope().is_none(),
        "Query bodies cannot seal as issued operation reads"
    );
    let rows = result
        .observed_sources()
        .iter()
        .map(|source| body(&world, source, &request))
        .collect::<Vec<_>>();
    let mut offset = 0;
    for cohort in full.aggregates() {
        let count = cohort.native().len();
        assert!(!Arc::ptr_eq(&rows[offset], &full));
        for row in &rows[offset..offset + count] {
            assert!(Arc::ptr_eq(row, &rows[offset]));
            assert!(Arc::ptr_eq(&row.aggregates()[0], cohort));
        }
        offset += count;
    }
    assert!(!Arc::ptr_eq(&rows[0], rows.last().unwrap()));
    let result = read::run(&world, Of::reference(), 0);
    let full = body(&world, result.result_set_observation().source(), &request);
    assert_eq!(full.aggregates().len(), 1);
    for source in result.observed_sources() {
        assert!(Arc::ptr_eq(&body(&world, source, &request), &full));
    }
}
#[test]
fn query_component_currentness_includes_coupling_even_when_native_membership_returns() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = super::super::super::decision::identity::install();
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let originals = [0, 3].map(|root| {
        let result = read::run(&world, Of::reference(), root);
        body(&world, result.result_set_observation().source(), &request)
    });
    let winner = usize::from(
        originals[0].aggregates()[0].native().anchor()
            > originals[1].aggregates()[0].native().anchor(),
    );
    let original = &originals[winner];
    for key in [249, 250] {
        let edit = super::super::super::program::swap_successors(&world, &request, 0, 3);
        assert!(matches!(
            super::super::super::commit(&world, &request, edit, key),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
    }
    let fresh = read::run(&world, Of::reference(), if winner == 0 { 0 } else { 3 });
    let fresh = body(&world, fresh.result_set_observation().source(), &request);
    assert_eq!(
        original.aggregates()[0].native().identity(),
        fresh.aggregates()[0].native().identity()
    );
    assert_eq!(
        original.aggregates()[0]
            .native()
            .members()
            .collect::<Vec<_>>(),
        fresh.aggregates()[0].native().members().collect::<Vec<_>>()
    );
    let lease = super::super::currentness::snapshot(&world, &execution);
    lease.handle().with_runtime(&execution, |runtime| {
        assert!(original.aggregates()[0]
            .remains_equal(runtime, &execution, lease.snapshot(), None)
            .unwrap());
        assert!(!original
            .remains_equal(runtime, &execution, lease.snapshot(), None)
            .unwrap());
    });
}
#[test]
fn empty_query_census_retains_negative_guard_and_ignores_unrelated_field_changes() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = super::super::super::decision::identity::install();
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let result = read::run(&world, Empty::reference(), 0);
    let original = body(&world, result.result_set_observation().source(), &request);
    assert!(original.aggregates().is_empty());
    for (root, key, expected) in [
        (6, "query-components-unrelated", true),
        (0, "query-components-guard", false),
    ] {
        let (candidate, idempotency) =
            super::super::super::decision::currentness::prepare(&world, &request, root, key);
        assert!(matches!(
            super::super::super::decision::currentness::publish(
                &world,
                &request,
                candidate,
                idempotency
            ),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        let lease = super::super::currentness::snapshot(&world, &execution);
        assert_eq!(
            lease.handle().with_runtime(&execution, |runtime| original
                .remains_equal(runtime, &execution, lease.snapshot(), None)
                .unwrap()),
            expected
        );
    }
}
