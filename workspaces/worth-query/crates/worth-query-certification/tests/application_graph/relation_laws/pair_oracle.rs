//! An independent physical multigraph oracle; it never reads production law policy.
use super::{installation::*, outcome::*, read, schema::*};
use std::collections::BTreeMap;
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

#[derive(Clone, Copy)]
enum Law {
    Symmetric,
    DirectedUnique,
    NormalizedUnique,
    SymmetricDirectedUnique,
}
type Bag = BTreeMap<(u64, u64), usize>;

fn allowed(graph: &Bag, law: Law) -> bool {
    if matches!(law, Law::Symmetric | Law::SymmetricDirectedUnique)
        && graph
            .keys()
            .any(|&(source, target)| !graph.contains_key(&(target, source)))
    {
        return false;
    }
    if matches!(law, Law::DirectedUnique | Law::SymmetricDirectedUnique)
        && graph.values().any(|count| *count > 1)
    {
        return false;
    }
    if matches!(law, Law::NormalizedUnique) {
        let mut normalized = BTreeMap::new();
        for (&(source, target), &count) in graph {
            *normalized
                .entry((source.min(target), source.max(target)))
                .or_insert(0usize) += count;
        }
        if normalized.values().any(|count| *count > 1) {
            return false;
        }
    }
    true
}

fn exercise<L: LawPolicy>(law: Law) {
    let world = install::<L>(4, &[]);
    let mut committed_graph = Bag::new();
    let mut seed = 0x9181_2210_a05du64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (seed >> 32) % 4
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
        let (added, removed) = match serial % 6 {
            0 => (vec![(a, b)], vec![]),
            1 => (pair(a, b), vec![]),
            2 => (vec![], vec![(a, b)]),
            3 => (vec![], pair(a, b)),
            4 => (vec![(a, c)], vec![(a, b)]),
            _ => (pair(a, c), pair(a, b)),
        };
        let mut proposed = committed_graph.clone();
        for pair in &removed {
            proposed.remove(pair);
        }
        for pair in &added {
            *proposed.entry(*pair).or_default() += 1;
        }
        let actual = world.replace_edges(serial, &added, &removed);
        if allowed(&proposed, law) {
            assert!(matches!(actual, WorthQueryApplicationMutationOutcome::Committed { .. }),
                "edit {serial}; added={added:?}, removed={removed:?}, before={committed_graph:?}, proposed={proposed:?}; actual={actual:?}");
            committed_graph = proposed;
            accepted += 1;
        } else {
            assert!(denied(actual).relation_law().is_some());
            refused += 1;
        }
        assert_eq!(
            read::committed(&world).multiplicity,
            committed_graph,
            "physical graph at edit {serial}"
        );
    }
    assert!(
        accepted > 0 && refused > 0,
        "the sequence must cross both boundaries"
    );
    world.close();
}

#[test]
fn seeded_pairs_follow_independent_symmetry_meaning() {
    exercise::<Symmetric>(Law::Symmetric);
}
#[test]
fn seeded_pairs_follow_independent_directed_uniqueness_meaning() {
    exercise::<DirectedUnique>(Law::DirectedUnique);
}
#[test]
fn seeded_pairs_follow_independent_normalized_uniqueness_meaning() {
    exercise::<NormalizedUnique>(Law::NormalizedUnique);
}
#[test]
fn seeded_pairs_follow_independent_composed_meaning() {
    exercise::<SymmetricDirectedUnique>(Law::SymmetricDirectedUnique);
}
