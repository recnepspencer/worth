//! Query observes actual walk acquisitions, independently of unrelated members.
use super::probe::Probe;
use super::*;
use crate::facade::application_invariants::RelationDirection;
use std::sync::{Arc, Mutex};

type Report = (usize, usize, [usize; 3]);
fn read_graph(graph: world::Graph, key: u8) -> Report {
    let reports = Arc::new(Mutex::new(Vec::new()));
    let observed = reports.clone();
    let probe = Probe::new(move |context, scope, rule| {
        let before = context.reads();
        let walk = context
            .proposed()
            .walk(&rule.next, RelationDirection::Forward, scope.node(0))
            .unwrap();
        let after = context.reads();
        let reads = [
            after.adjacency_lists_read() - before.adjacency_lists_read(),
            after.adjacency_edges_inspected() - before.adjacency_edges_inspected(),
            after.endpoint_records_read() - before.endpoint_records_read(),
        ];
        assert_eq!(
            reads[0],
            walk.len(),
            "one actual acquired adjacency per visited entity"
        );
        assert!(reads[1] > 0 && reads[2] > 0);
        assert_eq!(
            after.field_reads(),
            before.field_reads(),
            "walking does not load application fields"
        );
        let distance = walk.iter().map(|s| s.distance()).max().unwrap();
        observed.lock().unwrap().push((walk.len(), distance, reads));
    });
    let world = world::install_with_probe(graph, probe.clone());
    let request = super::super::super::fixture::live_scope();
    let program = program::set_active(&world, &request, true);
    probe.arm();
    assert!(matches!(
        commit(&world, &request, program, key),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let reports = reports.lock().unwrap();
    let report = *reports
        .first()
        .expect("an installed diagnostic callback observed actual reads");
    assert!(reports.iter().all(|other| *other == report));
    report
}

#[test]
fn a_thousand_step_installed_walk_completes_without_a_hidden_depth_cap() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let report = read_graph(
        world::Graph {
            nodes: 1001,
            edges: (0..1000).map(|key| (key, key + 1)).collect(),
            active: false,
            blocked: vec![],
        },
        218,
    );
    assert_eq!((report.0, report.1), (1001, 1000));
}

#[test]
fn adding_ten_thousand_unrelated_nodes_does_not_add_walk_acquisitions() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let graph = |unrelated: usize| world::Graph {
        nodes: 4 + unrelated,
        edges: vec![(0, 1), (1, 2), (2, 3)],
        active: false,
        blocked: vec![],
    };
    let small = read_graph(graph(10), 219);
    let large = read_graph(graph(10_000), 220);
    assert_eq!((small.0, small.1), (4, 3));
    assert_eq!(
        small, large,
        "measured context.reads delta around the same complete walk"
    );
}
