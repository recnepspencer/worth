use super::*;

fn diamond() -> (SignalGraph, [NodeId; 5]) {
    let mut graph = SignalGraph::new();
    let nodes = std::array::from_fn(|_| graph.create_node());
    let [producer, left, right, join, tail] = nodes;
    for (consumer, producers) in [
        (left, vec![producer]),
        (right, vec![producer]),
        (join, vec![left, right]),
        (tail, vec![join]),
    ] {
        graph.transition_node_clean(consumer).unwrap();
        graph
            .install_node_dependency_revalidation(consumer, producers.clone(), false)
            .unwrap();
        graph.replace_pending_revalidation_waiters(consumer, &[], &producers);
        assert_eq!(graph.get_state(consumer).unwrap(), NodeState::MaybeStale);
    }
    let (fork, _) = graph.fork_persistent();
    (fork, nodes)
}

#[test]
fn waiter_draft_resolves_diamond_without_writing_live_nodes_or_index() {
    let (graph, [producer, left, right, join, tail]) = diamond();
    let original_index = graph
        .topology
        .pending_revalidation_waiters
        .fork_storage_identity();
    let original_hot = graph.arena.hot.clone();
    let original_warm = graph.arena.warm.clone();
    // Generous measurement allowance includes physical index edits; the exact
    // and one-short twins below still establish the actual admission boundary.
    let mut work = Work::new(100_000);
    let prepared = graph
        .prepare_pending_revalidation_resolution(producer, BTreeMap::new(), &mut work)
        .unwrap();
    for consumer in [left, right, join, tail] {
        assert_eq!(prepared.nodes[&consumer].state, NodeState::Clean);
        assert!(prepared.nodes[&consumer].pending.is_none());
        assert_eq!(graph.get_state(consumer).unwrap(), NodeState::MaybeStale);
        assert!(graph.node_pending_revalidation(consumer).unwrap().is_some());
    }
    for source in [producer, left, right, join] {
        assert!(prepared.buckets[&source].is_empty());
    }
    let exact = graph
        .prepare_pending_revalidation_resolution(
            producer,
            BTreeMap::new(),
            &mut Work::new(work.visits()),
        )
        .unwrap();
    assert_eq!(exact.buckets, prepared.buckets);
    assert!(exact.nodes[&join].pending.is_none());
    assert!(matches!(
        graph.prepare_pending_revalidation_resolution(
            producer,
            BTreeMap::new(),
            &mut Work::new(work.visits() - 1)
        ),
        Err(PendingRevalidationPreparationDenial::Storage(
            RetainedStoragePreparationDenial::WorkExhausted { .. }
        ))
    ));
    assert!(graph
        .topology
        .pending_revalidation_waiters
        .ptr_eq(&original_index));
    assert!(graph.arena.hot.shares_storage_with(&original_hot));
    assert!(graph.arena.warm.shares_storage_with(&original_warm));
}

#[test]
fn projected_new_cause_and_structural_obligation_stop_stability_propagation() {
    for structural in [false, true] {
        let (graph, [producer, left, right, join, tail]) = diamond();
        let pending = if structural {
            Some(PendingDependencyRevalidation::structural(
                graph.node_dependency_revision(right).unwrap(),
                [producer],
            ))
        } else {
            graph.node_pending_revalidation(right).unwrap().cloned()
        };
        let projected = PendingRevalidationNodeProjection {
            state: if structural {
                NodeState::MaybeStale
            } else {
                NodeState::Dirty
            },
            pending,
            has_pending_causes: !structural,
            has_direct_basis: false,
            dirty_aspects: AspectMask::EMPTY,
        };
        let prepared = graph
            .prepare_pending_revalidation_resolution(
                producer,
                BTreeMap::from([(right, projected)]),
                &mut Work::new(100_000),
            )
            .unwrap();
        assert_eq!(prepared.nodes[&left].state, NodeState::Clean);
        assert_ne!(prepared.nodes[&right].state, NodeState::Clean);
        assert_eq!(prepared.nodes[&join].state, NodeState::MaybeStale);
        assert_eq!(
            prepared.nodes[&join]
                .pending
                .as_ref()
                .unwrap()
                .unresolved_producers(),
            &[right]
        );
        assert!(!prepared.nodes.contains_key(&tail));
        if structural {
            let pending = prepared.nodes[&right].pending.as_ref().unwrap();
            assert!(pending.is_resolved());
            assert!(pending.requires_structural_recompute());
        } else {
            assert!(prepared.nodes[&right].pending.is_none());
        }
        assert_eq!(
            graph
                .node_pending_revalidation(join)
                .unwrap()
                .unwrap()
                .unresolved_producers(),
            &[left, right]
        );
    }
}

#[test]
fn stale_candidates_consume_work_and_pruning_stays_in_the_draft() {
    let (mut graph, [producer, _, _, _, _]) = diamond();
    let stale = graph.create_node();
    // Membership can outlive the pending obligation. Use the same membership
    // owner as ordinary registration to exercise its stale-query contract.
    graph.replace_pending_revalidation_waiters(stale, &[], &[producer]);
    let mut baseline_work = Work::new(100_000);
    let prepared = graph
        .prepare_pending_revalidation_resolution(producer, BTreeMap::new(), &mut baseline_work)
        .unwrap();
    assert!(!prepared.buckets[&producer].contains(&stale));
    assert!(graph
        .topology
        .pending_revalidation_waiters
        .get(&producer)
        .unwrap()
        .contains(&stale));
    assert!(matches!(
        graph.prepare_pending_revalidation_resolution(producer, BTreeMap::new(), &mut Work::new(1)),
        Err(PendingRevalidationPreparationDenial::Storage(
            RetainedStoragePreparationDenial::WorkExhausted { .. }
        ))
    ));
    graph.replace_pending_revalidation_waiters(stale, &[producer], &[]);
    let mut without_stale = Work::new(100_000);
    graph
        .prepare_pending_revalidation_resolution(producer, BTreeMap::new(), &mut without_stale)
        .unwrap();
    assert!(baseline_work.visits() > without_stale.visits());
}

#[test]
fn waiter_sequence_overflow_denies_without_consuming_or_wrapping_work() {
    let mut work = Work::new(100);
    work.reserve_visits(7).unwrap();
    assert!(matches!(
        preparation_work::sequence_growth(&mut work, usize::MAX, 1),
        Err(PendingRevalidationPreparationDenial::Storage(
            RetainedStoragePreparationDenial::WorkExhausted {
                maximum_visits: 100
            }
        ))
    ));
    assert_eq!(work.visits(), 7);
}
