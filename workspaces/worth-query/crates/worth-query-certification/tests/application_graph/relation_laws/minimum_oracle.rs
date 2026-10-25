//! Independent physical multigraph model with distinct transitive reachability.
use super::{installation::*, outcome::*, read, schema::*};
use std::collections::{BTreeMap, BTreeSet};
use worth_query_host::facade::{
    application_invariants::EntityReference, primary_graph::WorthQueryRelationLawDenial,
};
type Bag = BTreeMap<(u64, u64), usize>;
fn reachable(graph: &Bag, source: u64) -> usize {
    let mut visited = BTreeSet::from([source]);
    let mut frontier = vec![source];
    while let Some(node) = frontier.pop() {
        for (&(from, target), &count) in graph {
            if count > 0 && from == node && visited.insert(target) {
                frontier.push(target);
            }
        }
    }
    visited.len() - 1
}
#[test]
fn seeded_edits_follow_an_independent_distinct_reachability_model() {
    let world = install::<MinimumTwo>(3, &[(0, 1), (1, 2), (2, 0)]);
    let identities = read::committed(&world).identities;
    let mut graph = Bag::from([((0, 1), 1), ((1, 2), 1), ((2, 0), 1)]);
    let mut seed = 0x9181_4d49_4e32_u64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (seed >> 32) % 3
    };
    let (mut accepted, mut refused) = (0, 0);
    for serial in 1..=72 {
        let (a, b, c) = (next(), next(), next());
        let pair = |x, y| {
            if x == y {
                vec![(x, y)]
            } else {
                vec![(x, y), (y, x)]
            }
        };
        let (added, removed) = if serial == 1 {
            (vec![], vec![(0, 1)])
        } else {
            match serial % 6 {
                0 => (vec![(a, b)], vec![]),
                1 => (pair(a, b), vec![]),
                2 => (vec![], vec![(a, b)]),
                3 => (vec![], pair(a, b)),
                4 => (vec![(a, c)], vec![(a, b)]),
                _ => (pair(a, c), pair(a, b)),
            }
        };
        let mut proposed = graph.clone();
        for edge in &removed {
            proposed.remove(edge);
        }
        for edge in &added {
            *proposed.entry(*edge).or_default() += 1;
        }
        let actual = world.replace_edges(serial, &added, &removed);
        if (0..3).all(|source| reachable(&proposed, source) >= 2) {
            committed(actual);
            graph = proposed;
            accepted += 1;
        } else {
            let refusal = denied(actual);
            let Some(WorthQueryRelationLawDenial::ConnectivityMinimum {
                source,
                reachable_target_count,
                minimum_reachable_targets,
                ..
            }) = refusal.relation_law()
            else {
                panic!("{refusal:?}")
            };
            let EntityReference::Existing(source) = source else {
                panic!("existing source")
            };
            let source_key = identities
                .iter()
                .find_map(|(key, identity)| (identity == source).then_some(*key))
                .unwrap();
            let expected = reachable(&proposed, source_key);
            assert!(expected < 2);
            assert_eq!(reachable_target_count, expected);
            assert_eq!(minimum_reachable_targets, 2);
            refused += 1;
        }
        assert_eq!(
            read::committed(&world).multiplicity,
            graph,
            "physical truth at {serial}"
        );
    }
    assert!(accepted > 0 && refused > 0);
    world.close();
}
