use crate::facade::SignalGraph;
use crate::tests::support::{GraphDependencyBatchExt, ASPECT_A, ASPECT_B};

#[test]
fn stress_edge_rewrites_across_reused_nodes() {
    let mut graph = SignalGraph::new();
    let roots: Vec<_> = (0..16).map(|_| graph.node().build()).collect();
    let leaves: Vec<_> = (0..64).map(|_| graph.node().build()).collect();

    for round in 0..40 {
        for (index, &leaf) in leaves.iter().enumerate() {
            let root = roots[(index + round) % roots.len()];
            let aspect = if (index + round) % 2 == 0 {
                ASPECT_A
            } else {
                ASPECT_B
            };
            let _ = graph.drop_dependency(leaf, roots[index % roots.len()], ASPECT_A);
            let _ = graph.drop_dependency(leaf, roots[index % roots.len()], ASPECT_B);
            graph.append_dependency(leaf, root, aspect).unwrap();
        }
    }

    for &root in &roots {
        for &subscriber in graph.subscribers_of(root).unwrap() {
            assert!(graph
                .dependencies_of(subscriber)
                .unwrap()
                .iter()
                .any(|edge| edge.source() == root));
        }
    }
}
