use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::node::NodeState;
use crate::data::output::PartitionSubscription;
use crate::data::proof::invalidation::binding::{OutputCommitOrdinal, ResolvedDependencyCause};
use crate::data::proof::PartitionScopeSet;
use crate::tests::support::GraphDependencyBatchExt;
use crate::tests::support::{ASPECT_A, ASPECT_B};

#[test]
fn rejected_cycle_preserves_topology_revision_and_cause_state() -> Result<(), SignalError> {
    let mut graph = SignalGraph::new();
    let source = graph.create_node();
    let middle = graph.create_node();
    let leaf = graph.create_node();
    graph.set_dependencies(middle, [DependencyEdge::new(source, ASPECT_A)])?;
    graph.set_dependencies(leaf, [DependencyEdge::new(middle, ASPECT_B)])?;

    let cause = ResolvedDependencyCause::new(
        graph.runtime_instance_id(),
        leaf,
        graph.dependency_revision(leaf)?,
        middle,
        ASPECT_B,
        None,
        0,
        OutputCommitOrdinal(1),
        1,
        PartitionScopeSet::default(),
    );
    graph.inject_pending_causes_unchecked_for_test(leaf, [cause])?;
    let source_revision = graph.dependency_revision(source)?;
    let leaf_causes = graph.pending_causes(leaf)?.to_vec();
    let source_dependencies = graph.dependencies_of(source)?.to_vec();
    let leaf_subscribers = graph.subscribers_of(leaf)?.to_vec();

    assert!(graph
        .set_dependencies(source, [DependencyEdge::new(leaf, ASPECT_A)])
        .is_err());
    assert_eq!(graph.dependency_revision(source)?, source_revision);
    assert_eq!(graph.dependencies_of(source)?, source_dependencies);
    assert_eq!(graph.subscribers_of(leaf)?, leaf_subscribers);
    assert_eq!(graph.pending_causes(leaf)?, leaf_causes);
    graph.assert_bidirectional_consistency()?;
    Ok(())
}

#[test]
fn accepted_rewire_advances_revision_and_invalidates_old_cause_caches() -> Result<(), SignalError> {
    let mut graph = SignalGraph::new();
    let old_source = graph.create_node();
    let new_source = graph.create_node();
    let consumer = graph.create_node();
    graph.set_dependencies(consumer, [DependencyEdge::new(old_source, ASPECT_B)])?;
    let prior_revision = graph.dependency_revision(consumer)?;
    let cause = ResolvedDependencyCause::new(
        graph.runtime_instance_id(),
        consumer,
        prior_revision,
        old_source,
        ASPECT_B,
        None,
        0,
        OutputCommitOrdinal(1),
        1,
        PartitionScopeSet::default(),
    );
    graph.inject_pending_causes_unchecked_for_test(consumer, [cause])?;
    assert!(!graph.node_dirty_aspects(consumer)?.is_empty());

    graph.set_dependencies(consumer, [DependencyEdge::new(new_source, ASPECT_B)])?;

    assert_eq!(graph.dependency_revision(consumer)?.0, prior_revision.0 + 1);
    assert!(graph.pending_causes(consumer)?.is_empty());
    assert!(graph.node_dirty_aspects(consumer)?.is_empty());
    assert!(graph.node_dirty_scoped_aspects(consumer)?.is_empty());
    graph.assert_bidirectional_consistency()?;
    Ok(())
}

#[test]
fn component_rewire_preserves_revision_state_and_bidirectional_topology() -> Result<(), SignalError>
{
    let mut graph = SignalGraph::new();
    let old_source = graph.create_node();
    let new_source = graph.create_node();
    let consumer = graph.create_node();
    graph.set_dependencies(consumer, [DependencyEdge::new(old_source, ASPECT_A)])?;
    graph.transition_node_clean(consumer)?;
    let prior_revision = graph.dependency_revision(consumer)?;

    graph.set_dependencies(consumer, [DependencyEdge::new(new_source, ASPECT_B)])?;

    let dependencies = graph.dependencies_of(consumer)?;
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].source(), new_source);
    assert_eq!(dependencies[0].aspect(), ASPECT_B);
    assert!(graph.subscribers_of(old_source)?.is_empty());
    assert_eq!(graph.subscribers_of(new_source)?, &[consumer]);
    assert_eq!(graph.dependency_revision(consumer)?.0, prior_revision.0 + 1);
    assert_eq!(graph.get_state(consumer)?, NodeState::MaybeStale);
    let pending = graph
        .pending_dependency_revalidation(consumer)?
        .expect("a topology rewrite must retain structural revalidation authority");
    assert_eq!(
        pending.dependency_revision(),
        graph.dependency_revision(consumer)?
    );
    assert!(pending.requires_structural_recompute());
    graph.assert_bidirectional_consistency()?;
    Ok(())
}

#[test]
fn batched_cycle_is_rejected_against_the_complete_proposed_topology() -> Result<(), SignalError> {
    let mut graph = SignalGraph::new();
    let left = graph.create_node();
    let right = graph.create_node();
    let left_revision = graph.dependency_revision(left)?;
    let right_revision = graph.dependency_revision(right)?;
    let edit = crate::data::proof::DependencyBatchEdit::from_pairs([
        (left, vec![DependencyEdge::new(right, ASPECT_A)]),
        (right, vec![DependencyEdge::new(left, ASPECT_B)]),
    ]);

    assert!(graph.apply_dependency_batch_edit(&edit).is_err());
    assert!(graph.dependencies_of(left)?.is_empty());
    assert!(graph.dependencies_of(right)?.is_empty());
    assert!(graph.subscribers_of(left)?.is_empty());
    assert!(graph.subscribers_of(right)?.is_empty());
    assert_eq!(graph.dependency_revision(left)?, left_revision);
    assert_eq!(graph.dependency_revision(right)?, right_revision);
    graph.assert_bidirectional_consistency()?;
    Ok(())
}

#[test]
fn retired_upstream_on_unrelated_path_does_not_reject_lawful_attach() -> Result<(), SignalError> {
    let mut graph = SignalGraph::new();
    let retired = graph.create_node();
    let middle = graph.create_node();
    let leaf = graph.create_node();
    graph.set_dependencies(middle, [DependencyEdge::new(retired, ASPECT_A)])?;
    graph.unregister_node(retired)?;
    graph.inject_retired_dependency_for_test(middle, retired, ASPECT_A)?;

    graph.set_dependencies(leaf, [DependencyEdge::new(middle, ASPECT_B)])?;

    assert_eq!(graph.dependencies_of(leaf)?[0].source(), middle);
    graph.assert_bidirectional_consistency()?;
    Ok(())
}

#[test]
fn rejected_scoped_cycles_do_not_grow_the_partition_interner() -> Result<(), SignalError> {
    let mut graph = SignalGraph::new();
    let upstream = graph.create_node();
    let downstream = graph.create_node();
    graph.set_dependencies(downstream, [DependencyEdge::new(upstream, ASPECT_A)])?;
    let baseline = graph.observe().metrics().partition_interner_size;

    for detail in ["1y", "2y", "5y"] {
        let rejected = DependencyEdge::with_partition_scope(
            downstream,
            ASPECT_B,
            PartitionSubscription::partition_and_detail("rates", detail),
        );
        assert!(graph.set_dependencies(upstream, [rejected]).is_err());
    }

    assert_eq!(graph.observe().metrics().partition_interner_size, baseline);
    Ok(())
}

#[test]
fn legacy_dependency_helpers_share_production_rewire_and_cycle_authority() -> Result<(), SignalError>
{
    fn graph_for_rewire() -> Result<
        (
            SignalGraph,
            crate::data::handle::NodeId,
            crate::data::handle::NodeId,
            crate::data::handle::NodeId,
        ),
        SignalError,
    > {
        let mut graph = SignalGraph::new();
        let old_source = graph.create_node();
        let new_source = graph.create_node();
        let consumer = graph.create_node();
        graph.set_dependencies(consumer, [DependencyEdge::new(old_source, ASPECT_A)])?;
        Ok((graph, old_source, new_source, consumer))
    }

    let (mut production, old_source, new_source, consumer) = graph_for_rewire()?;
    let (mut helper, helper_old, helper_new, helper_consumer) = graph_for_rewire()?;
    production.set_dependencies(consumer, [DependencyEdge::new(new_source, ASPECT_A)])?;
    helper.rewire_dependency(helper_consumer, helper_old, helper_new, ASPECT_A)?;
    assert_eq!(
        production.dependency_revision(consumer)?.0,
        helper.dependency_revision(helper_consumer)?.0
    );
    assert_eq!(
        production.get_state(consumer)?,
        helper.get_state(helper_consumer)?
    );
    assert_eq!(
        production
            .pending_dependency_revalidation(consumer)?
            .map(|pending| pending.requires_structural_recompute()),
        helper
            .pending_dependency_revalidation(helper_consumer)?
            .map(|pending| pending.requires_structural_recompute())
    );
    assert_eq!(production.dependencies_of(consumer)?.len(), 1);
    assert_eq!(helper.dependencies_of(helper_consumer)?.len(), 1);
    assert_ne!(old_source, new_source);

    let mut cycle_graph = SignalGraph::new();
    let left = cycle_graph.create_node();
    let right = cycle_graph.create_node();
    cycle_graph.append_dependency(right, left, ASPECT_A)?;
    let revision = cycle_graph.dependency_revision(left)?;
    assert!(cycle_graph
        .append_dependency(left, right, ASPECT_B)
        .is_err());
    assert_eq!(cycle_graph.dependency_revision(left)?, revision);
    assert!(cycle_graph.dependencies_of(left)?.is_empty());
    Ok(())
}

#[test]
fn checkpoint_restore_rebuilds_graph_local_partition_tokens() -> Result<(), SignalError> {
    let mut graph = SignalGraph::new();
    let source = graph.create_node();
    let consumer = graph.create_node();
    graph.set_dependencies(
        consumer,
        [DependencyEdge::partition_detail(
            source, ASPECT_A, "rates", "2y",
        )],
    )?;
    assert_eq!(graph.observe().metrics().partition_interner_size, 2);

    let authority = graph.capture_checkpoint_authority();
    let restored = SignalGraph::restore_from_checkpoint_authority(&authority)?;

    assert_eq!(restored.observe().metrics().partition_interner_size, 2);
    assert!(restored.dependencies_of(consumer)?[0]
        .interned_scope()
        .is_some());
    Ok(())
}
