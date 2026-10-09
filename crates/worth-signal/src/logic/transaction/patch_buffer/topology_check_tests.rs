use super::SparsePatchBuffer;
use crate::data::dependency::{CanonicalDependencies, DependencyEdge};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::tests::support::ASPECT_A;

#[derive(Clone, Copy, Debug)]
enum AutomaticSite {
    SingleReconcile,
    BatchReconcile,
    MembershipRepair,
    RetirementSources,
    RetirementSubscribers,
    PatchRollback,
    PatchRollbackAndClear,
}

#[test]
fn automatic_local_checks_detect_corruption_at_every_mutation_site() -> Result<(), SignalError> {
    for site in [
        AutomaticSite::SingleReconcile,
        AutomaticSite::BatchReconcile,
        AutomaticSite::MembershipRepair,
        AutomaticSite::RetirementSources,
        AutomaticSite::RetirementSubscribers,
        AutomaticSite::PatchRollback,
        AutomaticSite::PatchRollbackAndClear,
    ] {
        let mut fixture = corrupted_fixture(site)?;
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_mutation(site, &mut fixture).expect("mutation should reach its automatic check");
        }))
        .expect_err("automatic topology check must reject the touched corruption");
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .expect("topology panic should have a message");
        assert!(
            message.contains("topology inconsistency"),
            "{site:?}: {message}"
        );
        assert!(message.contains(&fixture.edge), "{site:?}: {message}");
    }
    Ok(())
}

struct Fixture {
    graph: SignalGraph,
    nodes: [NodeId; 4],
    patches: SparsePatchBuffer,
    edge: String,
}

fn corrupted_fixture(site: AutomaticSite) -> Result<Fixture, SignalError> {
    let mut graph = SignalGraph::new();
    assert!(graph.topology_debug_asserts_enabled());
    let nodes = std::array::from_fn(|_| graph.create_node());
    let [root, source, node, subscriber] = nodes;
    for (consumer, dependency) in [(source, root), (node, source), (subscriber, node)] {
        graph.set_dependencies(consumer, [DependencyEdge::new(dependency, ASPECT_A)])?;
    }
    let mut patches = SparsePatchBuffer::new();
    patches.stage_original(&graph, node)?;
    let (missing_source, missing_subscriber) = match site {
        AutomaticSite::MembershipRepair => {
            // Repair removes source -> subscriber; it does not repair the opposite missing node -> subscriber membership.
            graph.set_subscribers_sorted(source, &[node, subscriber])?;
            graph.set_subscribers_sorted(node, &[])?;
            (node, subscriber)
        }
        AutomaticSite::RetirementSources => {
            graph.set_subscribers_sorted(root, &[])?;
            (root, source)
        }
        AutomaticSite::RetirementSubscribers => {
            graph.set_dependencies(
                subscriber,
                [
                    DependencyEdge::new(root, ASPECT_A),
                    DependencyEdge::new(node, ASPECT_A),
                ],
            )?;
            graph.set_subscribers_sorted(root, &[source])?;
            (root, subscriber)
        }
        _ => {
            graph.set_subscribers_sorted(source, &[])?;
            (source, node)
        }
    };
    Ok(Fixture {
        graph,
        nodes,
        patches,
        edge: format!("missing subscriber edge {missing_source} -> {missing_subscriber}"),
    })
}

fn run_mutation(site: AutomaticSite, fixture: &mut Fixture) -> Result<(), SignalError> {
    let [root, source, node, _] = fixture.nodes;
    let desired = CanonicalDependencies::new([
        DependencyEdge::new(root, ASPECT_A),
        DependencyEdge::new(source, ASPECT_A),
    ]);
    match site {
        AutomaticSite::SingleReconcile => {
            fixture
                .graph
                .reconcile_dependencies(node, desired.as_slice())?;
        }
        AutomaticSite::BatchReconcile => {
            fixture
                .graph
                .reconcile_dependencies_batch(&[(node, desired)])?;
        }
        AutomaticSite::MembershipRepair => {
            fixture
                .graph
                .reconcile_subscriber_membership_for_sources(&[source])?;
        }
        AutomaticSite::RetirementSources | AutomaticSite::RetirementSubscribers => {
            fixture.graph.unregister_node(node)?;
        }
        AutomaticSite::PatchRollback => {
            std::mem::take(&mut fixture.patches).rollback_from_packet(&mut fixture.graph)?;
        }
        AutomaticSite::PatchRollbackAndClear => {
            fixture.patches.rollback_and_clear(&mut fixture.graph)?;
        }
    }
    Ok(())
}
