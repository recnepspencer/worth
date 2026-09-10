use super::*;
use crate::data::dependency::DependencyEdge;
use crate::tests::support::{evaluate_on_demand, version_ab};

#[test]
fn conditional_application_preserves_installed_scoped_topology_through_warm_reuse() {
    let (mut graph, contract) = installed();
    let source = graph.node().build();
    evaluate_on_demand(&mut graph, source, &mut |_, _| Ok(version_ab(0, 3))).unwrap();
    graph
        .set_dependencies(
            contract.node(),
            [DependencyEdge::partition_detail(
                source,
                Aspect::new(1),
                "partition-λ",
                "detail|=:",
            )],
        )
        .unwrap();
    let edges = graph
        .current_runtime_dependencies_of(contract.node())
        .unwrap()
        .to_vec();
    assert!(edges[0].interned_scope().is_some());
    let dependency_set = graph.node_dependency_ids(contract.node()).unwrap().0;
    let revision = graph.node_dependency_revision(contract.node()).unwrap();
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let mut computes = 0;
    for (attempt, forced, expected) in [
        (1, true, SignalConditionalDecisionClass::ComputedChanged),
        (
            2,
            true,
            SignalConditionalDecisionClass::ComputedRevertedClean,
        ),
        (
            3,
            false,
            SignalConditionalDecisionClass::DependencyUnchanged,
        ),
    ] {
        let request =
            SignalConditionalExecutionRequest::new(&contract, "source", "scoped", attempt);
        let request = if forced {
            request.force_on_demand()
        } else {
            request
        };
        let (decision, observation, rejected) = partition
            .execute_conditional(
                &mut graph,
                request,
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || {
                    computes += 1;
                    Ok(output(9))
                },
            )
            .unwrap()
            .into_parts();
        let decision = decision.unwrap();
        assert_eq!(decision.class(), expected);
        assert_eq!(decision.counters().runtime_dependency_edges_captured, 0);
        assert_eq!(decision.counters().compute_contacts, usize::from(forced));
        assert!(observation.is_ok());
        assert!(rejected.is_none());
        partition
            .execute(&mut graph, |selected| {
                assert_eq!(
                    selected
                        .current_runtime_dependencies_of(contract.node())
                        .unwrap(),
                    edges
                );
                assert_eq!(
                    selected.node_dependency_ids(contract.node()).unwrap().0,
                    dependency_set
                );
                assert_eq!(
                    selected.node_dependency_revision(contract.node()).unwrap(),
                    revision
                );
                let snapshot = selected.get_dep_snapshot(contract.node()).unwrap();
                assert_eq!(snapshot.entries().len(), 1);
                assert_eq!(snapshot.entries()[0].cached_version, 3);
                assert_eq!(snapshot.entries()[0].scope.as_ref(), edges[0].scope_ref());
            })
            .unwrap();
    }
    assert_eq!(computes, 2);
}

#[test]
fn conditional_refresh_still_removes_retired_edges_before_application() {
    let (mut graph, contract) = installed();
    let source = graph.node().build();
    graph
        .set_dependencies(
            contract.node(),
            [DependencyEdge::new(source, Aspect::new(1))],
        )
        .unwrap();
    graph.unregister_node(source).unwrap();
    // Explicit stale-topology fault fixture using a genuinely retired handle.
    graph
        .inject_retired_dependency_for_test(contract.node(), source, Aspect::new(1))
        .unwrap();
    assert_eq!(
        graph
            .current_runtime_dependencies_of(contract.node())
            .unwrap()
            .len(),
        1
    );
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let (decision, observation, rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "source", "pruned", 1)
                .force_on_demand(),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Ok(output(9)),
        )
        .unwrap()
        .into_parts();
    assert_eq!(
        decision.unwrap().class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    assert!(observation.is_ok());
    assert!(rejected.is_none());
    partition
        .execute(&mut graph, |selected| {
            use crate::data::retained_storage::{
                RetainedStorageMeasurement, RetainedStoragePreparation,
            };
            let carried = [
                selected.arena.hot.prepared_retained_charge(),
                selected.arena.warm.prepared_retained_charge(),
                selected.arena.cold.prepared_retained_charge(),
            ];
            let measured = [
                selected
                    .arena
                    .hot
                    .retained_heap_charge(&mut RetainedStoragePreparation::new(100_000))
                    .unwrap(),
                selected
                    .arena
                    .warm
                    .retained_heap_charge(&mut RetainedStoragePreparation::new(100_000))
                    .unwrap(),
                selected
                    .arena
                    .cold
                    .retained_heap_charge(&mut RetainedStoragePreparation::new(100_000))
                    .unwrap(),
            ];
            assert_eq!(carried, measured.map(Ok));
            assert!(selected
                .current_runtime_dependencies_of(contract.node())
                .unwrap()
                .is_empty());
            assert!(selected
                .get_dep_snapshot(contract.node())
                .unwrap()
                .entries()
                .is_empty());
        })
        .unwrap();
}

#[test]
fn retired_edge_refresh_denial_precedes_node_and_topology_publication() {
    let (mut graph, contract) = installed();
    let source = graph.node().build();
    graph
        .set_dependencies(
            contract.node(),
            [DependencyEdge::new(source, Aspect::new(1))],
        )
        .unwrap();
    graph.unregister_node(source).unwrap();
    graph
        .inject_retired_dependency_for_test(contract.node(), source, Aspect::new(1))
        .unwrap();
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    partition
        .execute(&mut graph, |selected| {
            let node = contract.node();
            let before_revision = selected.node_dependency_revision(node).unwrap();
            let before_edges = selected
                .current_runtime_dependencies_of(node)
                .unwrap()
                .to_vec();
            let before_state = selected.get_state(node).unwrap();
            let ledger = selected
                .arena
                .retained_node_ledger
                .as_ref()
                .unwrap()
                .clone();
            ledger.close();
            let usage = ledger.usage();
            let result = selected.refresh_runtime_dependencies_with_work(
                node,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary,
            );
            assert_eq!(
                result,
                Err(crate::data::error::SignalError::EvaluationStorageUnavailable)
            );
            assert_eq!(
                selected.node_dependency_revision(node).unwrap(),
                before_revision
            );
            assert_eq!(
                selected.current_runtime_dependencies_of(node).unwrap(),
                before_edges
            );
            assert_eq!(selected.get_state(node).unwrap(), before_state);
            assert_eq!(ledger.usage(), usage);
        })
        .unwrap();
}

#[test]
fn topology_replacement_removes_the_previous_structural_waiter_membership() {
    let mut graph = crate::data::graph::SignalGraph::new();
    let previous = graph.node().build();
    let next = graph.node().build();
    let consumer = graph.node().build();
    graph
        .set_dependencies(consumer, [DependencyEdge::new(previous, Aspect::new(1))])
        .unwrap();
    assert!(graph
        .topology
        .pending_revalidation_waiters
        .get(&previous)
        .unwrap()
        .contains(&consumer));
    graph
        .set_dependencies(consumer, [DependencyEdge::new(next, Aspect::new(1))])
        .unwrap();
    assert!(!graph
        .topology
        .pending_revalidation_waiters
        .get(&previous)
        .is_some_and(|waiters| waiters.contains(&consumer)));
    assert!(graph
        .topology
        .pending_revalidation_waiters
        .get(&next)
        .unwrap()
        .contains(&consumer));
    let pending = graph.node_pending_revalidation(consumer).unwrap().unwrap();
    assert_eq!(pending.unresolved_producers(), &[next]);
    assert_eq!(
        pending.dependency_revision(),
        graph.node_dependency_revision(consumer).unwrap()
    );
}
