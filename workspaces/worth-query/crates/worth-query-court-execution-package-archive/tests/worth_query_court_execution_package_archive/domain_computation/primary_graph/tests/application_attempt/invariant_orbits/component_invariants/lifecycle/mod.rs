//! Empty census is genuine negative truth and cannot suppress later scheduling.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::component_invariants::program::change_spare;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
#[test]
fn empty_component_count_reprepares_after_real_member_creation_and_deletion() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let stage = Arc::new(AtomicUsize::new(1));
    let current = stage.clone();
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let observer = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe::<false>::new(move |context, _, rule| {
        let proposed = context.proposed().components(&rule.spares).unwrap();
        let committed = context.committed().components(&rule.spares).unwrap();
        let (before, after) = if current.load(Ordering::SeqCst) == 1 {
            (0, 1)
        } else {
            (1, 0)
        };
        assert_eq!(committed.len(), before);
        assert_eq!(proposed.len(), after);
        assert_eq!(proposed.is_empty(), after == 0);
        assert_eq!(
            context.proposed().component_count(&rule.spares).unwrap(),
            after
        );
        for group in proposed.iter() {
            assert_eq!(group.len(), 1);
            let member = group.members().next().unwrap();
            assert_eq!(
                context
                    .proposed()
                    .component_of(&rule.spares, &member)
                    .unwrap()
                    .identity(),
                group.identity()
            );
        }
        seen.fetch_add(1, Ordering::SeqCst);
    })
    .complete_components();
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::install_with_probe(
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::Graph {
            nodes: 1,
            edges: vec![],
            active: false,
            blocked: vec![],
        },
        observer.clone(),
    );
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    observer.arm();
    for (next, create) in [(1, true), (2, false)] {
        stage.store(next, Ordering::SeqCst);
        let before = calls.load(Ordering::SeqCst);
        let operation = change_spare(&world, &request, create);
        assert!(matches!(
            commit(&world, &request, operation, 188 + next as u8),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        assert!(
            calls.load(Ordering::SeqCst) > before,
            "genuine member create/delete schedules fresh preparation even after an empty census"
        );
    }
}
