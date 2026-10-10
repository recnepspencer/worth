use crate::facade::{mark_dirty, EvaluationRequestMode, SignalGraph};
use crate::tests::leased_execution::support::{authority, request};
use crate::tests::support::{version_ab, ASPECT_A};

use super::canonical_artifact_oracle::canonical_runtime_artifacts;
use super::executor_policy::{aggressive_parallel_runtime_policy, bounded_contract};

#[test]
fn second_output_preparation_failure_preserves_the_entire_epoch() {
    assert_epoch_unchanged(false);
}

#[test]
fn second_semantic_preparation_failure_preserves_outputs_and_diagnostics() {
    assert_epoch_unchanged(true);
}

fn assert_epoch_unchanged(semantic_fault: bool) {
    for workers in [1, 2, 4] {
        let mut graph = SignalGraph::new();
        graph.set_runtime_policy(aggressive_parallel_runtime_policy());
        let old_source = graph.node().with_contract(bounded_contract(&[])).build();
        let new_source = graph.node().with_contract(bounded_contract(&[])).build();
        let targets = std::array::from_fn::<_, 2, _>(|_| {
            graph
                .node()
                .with_contract(bounded_contract(&[old_source, new_source]))
                .build()
        });
        let shared = graph
            .node()
            .with_contract(bounded_contract(&targets))
            .build();
        let nodes = [old_source, new_source, targets[0], targets[1], shared];
        let bootstrap = graph
            .build_evaluation_plan(&nodes, EvaluationRequestMode::ForceOnDemand)
            .unwrap();
        graph
            .execute_prepared_plan(&bootstrap, &(), &|ctx| {
                let value = if ctx.node() == old_source {
                    10
                } else if ctx.node() == new_source {
                    20
                } else if ctx.node() == shared {
                    ctx.read_aspect_version(targets[0], ASPECT_A)?.get(ASPECT_A)
                        + ctx.read_aspect_version(targets[1], ASPECT_A)?.get(ASPECT_A)
                } else {
                    ctx.read_aspect_version(old_source, ASPECT_A)?.get(ASPECT_A)
                };
                Ok(ctx.finish(version_ab(value, 0)))
            })
            .unwrap();
        for target in targets {
            mark_dirty(&mut graph, target, ASPECT_A).unwrap();
        }
        let states = nodes.map(|node| graph.get_state(node).unwrap());
        let before = nodes.map(|node| {
            (
                graph.node_aspect_version(node).unwrap(),
                graph.dependencies_of(node).unwrap().to_vec(),
                graph.subscribers_of(node).unwrap().to_vec(),
                graph.get_dep_snapshot(node).unwrap().clone(),
                canonical_runtime_artifacts(&graph, node),
            )
        });
        let allocators = graph.diagnostics_state().lineage_allocator_state();
        let lineage = graph
            .diagnostics_state()
            .lineage_records()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let explanation = nodes.map(|node| graph.explanation_fact(node).cloned());
        let provenance = nodes.map(|node| graph.provenance_fact(node).cloned());
        let message = if semantic_fault {
            graph.fail_second_semantic_preparation_for_test();
            "injected second semantic preparation failure"
        } else {
            graph.fail_second_output_preparation_for_test();
            "injected second output preparation failure"
        };
        let lease = authority()
            .request_lease(request(workers, 2_000_000))
            .unwrap();
        let error = graph
            .evaluate_checked(
                &targets,
                EvaluationRequestMode::Default,
                &(),
                &|ctx| Ok(version_ab(ctx.read(new_source, ASPECT_A)?, 0)),
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
        assert_eq!(
            graph.diagnostics_state().lineage_allocator_state(),
            allocators
        );
        assert_eq!(
            graph
                .diagnostics_state()
                .lineage_records()
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            lineage
        );
        for ((node, expected), index) in nodes.into_iter().zip(before).zip(0..) {
            assert_eq!(graph.get_state(node).unwrap(), states[index]);
            assert_eq!(graph.explanation_fact(node), explanation[index].as_ref());
            assert_eq!(graph.provenance_fact(node), provenance[index].as_ref());
            assert_eq!(graph.node_aspect_version(node).unwrap(), expected.0);
            assert_eq!(graph.dependencies_of(node).unwrap(), expected.1);
            assert_eq!(graph.subscribers_of(node).unwrap(), expected.2);
            assert_eq!(graph.get_dep_snapshot(node).unwrap(), &expected.3);
            assert_eq!(canonical_runtime_artifacts(&graph, node), expected.4);
        }
    }
}
