use std::collections::{BTreeMap, BTreeSet};

use super::*;

/// Independent layer expansion over authored ordinal pairs, not native reads
/// or the production queue. Native identities enter only the final comparison.
fn model(
    count: usize,
    edges: &[(usize, usize, KindId)],
    roots: &[usize],
    incoming: bool,
) -> BTreeMap<usize, usize> {
    let mut distances = BTreeMap::new();
    let mut layer = roots.iter().copied().collect::<BTreeSet<_>>();
    let mut distance = 0;
    while !layer.is_empty() {
        for &entity in &layer {
            distances.insert(entity, distance);
        }
        let mut next = BTreeSet::new();
        for candidate in 0..count {
            if distances.contains_key(&candidate) {
                continue;
            }
            if edges.iter().any(|&(source, target, kind)| {
                kind == KindId(2)
                    && if incoming {
                        candidate == source && layer.contains(&target)
                    } else {
                        candidate == target && layer.contains(&source)
                    }
            }) {
                next.insert(candidate);
            }
        }
        layer = next;
        distance += 1;
    }
    distances
}

#[test]
fn complete_walk_matches_independent_bfs_and_retains_non_discovery_edges() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let mut seed = 0x9181_u64;
    for case in 0..12 {
        let count = 19;
        let mut edges = vec![
            (0, 0, KindId(2)),
            (0, 1, KindId(2)),
            (0, 2, KindId(2)),
            (1, 3, KindId(2)),
            (2, 3, KindId(2)),
            (3, 0, KindId(2)),
            (0, 18, KindId(3)),
        ];
        let mut pairs = edges.iter().copied().collect::<BTreeSet<_>>();
        for _ in 0..35 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let source = (seed >> 32) as usize % 17;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let target = (seed >> 32) as usize % 17;
            let edge = (source, target, KindId(2));
            if pairs.insert(edge) {
                edges.push(edge);
            }
        }
        if case % 2 == 0 {
            edges.reverse();
        }
        let graph = Graph::new(&execution, count, &edges);
        for incoming in [false, true] {
            for roots in [vec![0], vec![3, 0, 3, 17], vec![17, 0, 3]] {
                let distances = model(count, &edges, &roots, incoming);
                let seeds = roots
                    .iter()
                    .map(|&root| graph.entities[root])
                    .collect::<Vec<_>>();
                let walk = graph
                    .read(&execution, unbounded(), |view| {
                        if incoming {
                            view.walk_incoming_from(&seeds, KindId(2))
                        } else {
                            view.walk_outgoing_from(&seeds, KindId(2))
                        }
                    })
                    .unwrap();
                let mut expected = distances
                    .iter()
                    .map(|(&index, &depth)| (graph.entities[index], depth))
                    .collect::<Vec<_>>();
                expected.sort_by_key(|&(entity, depth)| (depth, entity));
                assert_eq!(visits(&walk), expected, "case {case}, incoming {incoming}");
                let expected_edges = edges
                    .iter()
                    .filter(|&&(source, target, kind)| {
                        kind == KindId(2)
                            && distances.contains_key(&if incoming { target } else { source })
                    })
                    .count();
                assert_eq!(walk.relations().len(), expected_edges);
                assert_eq!(
                    walk.relations()
                        .iter()
                        .map(|record| record.relation_id)
                        .collect::<BTreeSet<_>>()
                        .len(),
                    expected_edges
                );
                assert_eq!(walk.reads().adjacency_lists, distances.len());
                assert_eq!(walk.reads().entity_records, distances.len());
            }
        }
    }
}

#[test]
fn thousand_hops_complete_and_disconnected_growth_adds_no_reads() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let edges = (0..1000)
        .map(|index| (index, index + 1, KindId(2)))
        .collect::<Vec<_>>();
    let mut graph = Graph::new(&execution, 1001, &edges);
    let root = graph.entities[0];
    let walk = graph
        .read(&execution, unbounded(), |view| {
            view.walk_outgoing_from(&[root], KindId(2))
        })
        .unwrap();
    assert_eq!(
        visits(&walk),
        graph
            .entities
            .iter()
            .copied()
            .enumerate()
            .map(|(distance, entity)| (entity, distance))
            .collect::<Vec<_>>()
    );
    graph.add_entities(&execution, 10);
    let small = graph
        .read(&execution, unbounded(), |view| {
            view.walk_outgoing_from(&[root], KindId(2))
        })
        .unwrap();
    graph.add_entities(&execution, 9990);
    let large = graph
        .read(&execution, unbounded(), |view| {
            view.walk_outgoing_from(&[root], KindId(2))
        })
        .unwrap();
    assert_eq!(visits(&small), visits(&large));
    assert_eq!(small.relations(), large.relations());
    assert_eq!(walk.reads(), small.reads());
    assert_eq!(small.reads(), large.reads());
}
