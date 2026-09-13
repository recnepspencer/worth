use crate::facade::{
    Aspect, AsyncNodeRequestIntent, DependencyEdge, SignalError, SignalGraph, SignalRuntimePolicy,
};
use crate::tests::async_node_support::{async_node_capability_declaration, AsyncNodeTestRuntime};
use crate::tests::support::{evaluate, version_ab};

#[test]
fn async_upstream_work_denial_precedes_resource_admission_with_exact_twin() {
    for maximum in [1, 2] {
        let mut graph = SignalGraph::new();
        let leaf = graph.node().build();
        let middle = graph.node().build();
        let target = graph.node().build();
        graph
            .set_dependencies(middle, [DependencyEdge::new(leaf, Aspect::new(0))])
            .unwrap();
        graph
            .set_dependencies(target, [DependencyEdge::new(middle, Aspect::new(0))])
            .unwrap();
        evaluate(&mut graph, target, &mut |_, _| Ok(version_ab(1, 0))).unwrap();
        let mut runtime = AsyncNodeTestRuntime::builder(graph)
            .with_kernel_defaults()
            .runtime_policy(
                SignalRuntimePolicy::development().with_maximum_upstream_dependency_visits(maximum),
            )
            .build();
        runtime
            .declare_async_node_capability(async_node_capability_declaration(target))
            .unwrap();
        let result = runtime.admit_async_node_request(AsyncNodeRequestIntent::new(target));
        if maximum == 1 {
            assert!(matches!(
                result,
                Err(SignalError::UpstreamDependencyWorkExhausted { maximum_visits: 1 })
            ));
            assert_eq!(
                runtime.resource_runtime_summary().in_flight_request_count(),
                0
            );
        } else {
            assert!(result.unwrap().resource_admission().is_some());
            assert_eq!(
                runtime.resource_runtime_summary().in_flight_request_count(),
                1
            );
        }
    }
}
