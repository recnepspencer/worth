use super::*;

fn graph_with_waiters(waiter_count: u32) -> (SignalGraph, NodeId, Vec<NodeId>) {
    let mut graph = SignalGraph::new();
    let producer = graph.create_node();
    let consumers = (0..waiter_count)
        .map(|_| graph.create_node())
        .collect::<Vec<_>>();
    for &consumer in &consumers {
        graph
            .get_entry_mut(consumer)
            .expect("consumer is live")
            .mark_pending_dependency_revalidation([producer]);
        graph
            .topology
            .pending_revalidation_waiters
            .entry(producer)
            .or_default()
            .insert(consumer);
    }
    (graph, producer, consumers)
}

#[test]
fn unchanged_waiter_query_preserves_exact_shared_storage() -> Result<(), SignalError> {
    let (mut source, producer, consumers) = graph_with_waiters(64);
    let (mut destination, _) = source.fork_persistent();
    assert!(
        source
            .topology
            .pending_revalidation_waiters
            .ptr_eq(&destination.topology.pending_revalidation_waiters),
        "genuine graph fork must share the waiter index"
    );

    assert_eq!(
        destination.pending_revalidation_waiters(producer)?,
        consumers
    );
    assert!(
        source
            .topology
            .pending_revalidation_waiters
            .ptr_eq(&destination.topology.pending_revalidation_waiters),
        "an unchanged observation must not install a persistent overlay"
    );
    assert_eq!(source.pending_revalidation_waiters(producer)?, consumers);
    Ok(())
}

#[test]
fn stale_waiter_cleanup_changes_only_the_destination() -> Result<(), SignalError> {
    let (mut source, producer, consumers) = graph_with_waiters(4_096);
    let (mut destination, _) = source.fork_persistent();
    let (mut sibling, _) = source.fork_persistent();
    let stale = consumers[2_048];
    destination
        .get_entry_mut(stale)?
        .mark_pending_dependency_revalidation([]);

    let destination_waiters = destination.pending_revalidation_waiters(producer)?;
    let expected_destination = consumers
        .iter()
        .copied()
        .filter(|consumer| *consumer != stale)
        .collect::<Vec<_>>();
    assert_eq!(destination_waiters, expected_destination);
    assert_eq!(source.pending_revalidation_waiters(producer)?, consumers);
    assert_eq!(sibling.pending_revalidation_waiters(producer)?, consumers);
    assert_eq!(
        destination
            .topology
            .pending_revalidation_waiters
            .get(&producer)
            .expect("destination keeps the nonempty waiter set")
            .len(),
        consumers.len() - 1
    );
    assert_eq!(
        source
            .topology
            .pending_revalidation_waiters
            .get(&producer)
            .expect("source waiter set remains live")
            .len(),
        consumers.len()
    );
    Ok(())
}

#[test]
fn absent_and_empty_waiter_sets_return_empty_and_leave_no_index_entry() -> Result<(), SignalError> {
    let mut graph = SignalGraph::new();
    let producer = graph.create_node();
    assert!(graph.pending_revalidation_waiters(producer)?.is_empty());
    graph
        .topology
        .pending_revalidation_waiters
        .insert(producer, im::OrdSet::new());

    assert!(graph.pending_revalidation_waiters(producer)?.is_empty());
    assert!(!graph
        .topology
        .pending_revalidation_waiters
        .contains_key(&producer));
    Ok(())
}

#[test]
fn fully_stale_waiter_set_is_removed_without_touching_the_source() -> Result<(), SignalError> {
    let (mut source, producer, consumers) = graph_with_waiters(4_096);
    let (mut destination, _) = source.fork_persistent();
    for &consumer in &consumers {
        destination
            .get_entry_mut(consumer)?
            .mark_pending_dependency_revalidation([]);
    }

    let destination_waiters = destination.pending_revalidation_waiters(producer)?;
    assert!(destination_waiters.is_empty());
    assert_eq!(
        destination_waiters.capacity(),
        0,
        "an all-stale observation must not retain candidate-sized output storage"
    );
    assert!(!destination
        .topology
        .pending_revalidation_waiters
        .contains_key(&producer));
    assert_eq!(source.pending_revalidation_waiters(producer)?, consumers);
    Ok(())
}

#[test]
fn stale_dominant_waiter_result_retains_only_survivor_storage() -> Result<(), SignalError> {
    let (mut source, producer, consumers) = graph_with_waiters(4_096);
    let (mut destination, _) = source.fork_persistent();
    let (mut sibling, _) = source.fork_persistent();
    let survivor = *consumers.last().expect("fixture has one survivor");
    for &consumer in &consumers[..consumers.len() - 1] {
        destination
            .get_entry_mut(consumer)?
            .mark_pending_dependency_revalidation([]);
    }

    let destination_waiters = destination.pending_revalidation_waiters(producer)?;
    assert_eq!(destination_waiters, [survivor]);
    assert_eq!(
        destination_waiters.capacity(),
        destination_waiters.len(),
        "stale candidates must not determine retained result storage"
    );
    assert_eq!(
        destination
            .topology
            .pending_revalidation_waiters
            .get(&producer),
        Some(&im::ordset![survivor])
    );
    assert_eq!(source.pending_revalidation_waiters(producer)?, consumers);
    assert_eq!(sibling.pending_revalidation_waiters(producer)?, consumers);
    Ok(())
}
