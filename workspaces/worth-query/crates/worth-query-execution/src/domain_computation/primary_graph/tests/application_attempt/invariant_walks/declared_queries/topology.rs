//! Results, original shared body, actual origin and work are observed together.
use super::*;
use std::sync::Arc;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryRootPathTraversal, RelationDirection,
};

#[test]
fn complete_declared_paths_preserve_cycles_reverse_and_actual_origins() {
    let leaf = world::install(Graph {
        nodes: 1,
        edges: vec![],
        active: false,
        blocked: vec![],
    });
    let reflexive = read::run(&leaf, Forward::reference(), 0);
    assert_eq!(keys(&reflexive), [0]);
    let reads = reflexive.receipt().root_path_steps()[0].reads();
    assert_eq!(
        (
            reads.adjacency_lists_read(),
            reads.adjacency_edges_inspected(),
            reads.endpoint_records_read()
        ),
        (1, 0, 1)
    );
    let empty = read::run(&leaf, Composed::reference(), 0);
    assert!(empty.rows().is_empty());
    assert_eq!(empty.receipt().root_path_steps().len(), 2);
    assert_eq!(
        empty.receipt().root_path_steps()[1].reads(),
        WorthQueryInvariantProjectionWork::default(),
        "an empty frontier issues no Native closure read"
    );
    let world = diamond(6, vec![]);
    let forward = read::run(&world, Forward::reference(), 0);
    assert_eq!(keys(&forward), [0, 1, 2, 3, 4]);
    let reverse = read::run(&world, Reverse::reference(), 4);
    assert_eq!(keys(&reverse), [0, 1, 2, 3, 4]);
    for (result_steps, direction) in [
        (
            forward.receipt().root_path_steps(),
            RelationDirection::Forward,
        ),
        (
            reverse.receipt().root_path_steps(),
            RelationDirection::Reverse,
        ),
    ] {
        assert_eq!(result_steps.len(), 1);
        let step: crate::facade::primary_graph::WorthQueryApplicationQueryRootPathStepRead =
            result_steps[0];
        assert_eq!(
            (
                step.path_index(),
                step.step_index(),
                step.traversal(),
                step.direction()
            ),
            (0, 0, ApplicationQueryRootPathTraversal::Closure, direction)
        );
        assert_eq!(step.reads().adjacency_lists_read(), 5);
        assert_eq!(step.reads().adjacency_edges_inspected(), 7);
        assert_eq!(step.reads().endpoint_records_read(), 5);
    }
    let composed = read::run(&world, Composed::reference(), 0);
    assert_eq!(keys(&composed), [1, 2, 3, 4]);
    let selections: Vec<_> = composed
        .observed_sources()
        .iter()
        .map(|source| {
            (
                source.source_root(),
                source.footprint_for_test().root_selection.as_ref().unwrap(),
            )
        })
        .collect();
    let body = &selections[0].1.walks.as_ref().unwrap().values()[0];
    assert_eq!(body.key().roots().len(), 2);
    for (root, selection) in selections {
        assert!(Arc::ptr_eq(
            body,
            &selection.walks.as_ref().unwrap().values()[0]
        ));
        assert_eq!(
            selection.adjacencies.len(),
            1,
            "only the real one-hop predecessor adjacency is retained"
        );
        let visit = body
            .native()
            .visits()
            .iter()
            .find(|visit| visit.entity.entity_id == root)
            .unwrap();
        let expected = std::collections::BTreeSet::from([visit.root, root]);
        assert_eq!(
            selection
                .aspects
                .iter()
                .map(|aspect| aspect.entity)
                .collect::<std::collections::BTreeSet<_>>(),
            expected,
            "exactly the actual predecessor guard and this terminal guard are retained"
        );
    }
    let guarded = diamond(6, vec![1]);
    let guarded_result = read::run(&guarded, Composed::reference(), 0);
    assert_eq!(keys(&guarded_result), [2, 3, 4]);
    for source in guarded_result.observed_sources() {
        let selection = source.footprint_for_test().root_selection.as_ref().unwrap();
        let body = &selection.walks.as_ref().unwrap().values()[0];
        assert_eq!(
            body.key().roots().len(),
            1,
            "the prefix guard actually removes a Native root"
        );
        assert!(body
            .native()
            .visits()
            .iter()
            .all(|visit| visit.root == body.key().roots()[0]));
    }
}
#[test]
fn complete_declared_path_has_no_legacy_depth_or_unrelated_graph_scan() {
    let chain = world::install(Graph {
        nodes: 1001,
        edges: (0..1000).map(|n| (n, n + 1)).collect(),
        active: false,
        blocked: vec![],
    });
    let host = crate::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let (result, report) = read::execute_measured(
        &chain,
        &execution,
        &request,
        Forward::reference(),
        0,
        worth_execution::ExecutionWorkCeiling::new(None),
    );
    let result = result.unwrap();
    let envelope = report
        .charged_work()
        .checked_mul(4)
        .and_then(|n| n.checked_add(64))
        .unwrap();
    let (again, _) = read::execute_measured(
        &chain,
        &execution,
        &request,
        Forward::reference(),
        0,
        worth_execution::ExecutionWorkCeiling::new(Some(envelope)),
    );
    let again = again.unwrap();
    assert_eq!(
        result.observed_sources().len(),
        again.observed_sources().len()
    );
    for (first, second) in result
        .observed_sources()
        .iter()
        .zip(again.observed_sources())
    {
        assert!(
            first
                .output_source_epoch()
                .unwrap()
                .same_semantic_source(&second.output_source_epoch().unwrap()),
            "actual semantic source epochs are interned across real equal reads"
        );
    }
    assert_eq!(
        again.receipt().root_path_steps()[0]
            .reads()
            .endpoint_records_read(),
        1001,
        "the equal second body came from another actual Native walk"
    );
    assert_eq!(result.rows().len(), 1001);
    assert_eq!(keys(&result).last(), Some(&1000));
    let small = diamond(6, vec![]);
    let large = diamond(10006, vec![]);
    let a = read::run(&small, Forward::reference(), 0);
    let b = read::run(&large, Forward::reference(), 0);
    assert_eq!(keys(&a), keys(&b));
    assert_eq!(a.receipt().root_path_steps(), b.receipt().root_path_steps());
    assert_eq!(a.receipt().reads(), b.receipt().reads());
}
