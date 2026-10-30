//! Hostile/lawful twins and an independent small-graph reachability oracle.
pub use super::{installation::*, outcome::*, read, schema::*};
pub use std::collections::BTreeSet;
pub use worth_query_host::facade::{application_invariants::EntityReference, primary_graph::*};

pub fn closing_edge(
    denial: &WorthQueryApplicationCommitDenial,
    graph: &read::Graph,
    candidates: &[(u64, u64)],
) {
    let Some(WorthQueryRelationLawDenial::Acyclicity {
        contract_id,
        source: EntityReference::Existing(source),
        target: EntityReference::Existing(target),
        ..
    }) = denial.relation_law()
    else {
        panic!("original typed closing-edge identities required: {denial:?}");
    };
    assert_eq!(contract_id.as_str(), "Precedes:acyclic");
    assert!(candidates
        .iter()
        .any(|&(a, b)| graph.identities[&a] == *source && graph.identities[&b] == *target));
}

pub fn has_cycle(edges: &BTreeSet<(u64, u64)>) -> bool {
    edges.iter().any(|&(source, target)| {
        let mut frontier = vec![target];
        let mut seen = BTreeSet::new();
        while let Some(node) = frontier.pop() {
            if node == source {
                return true;
            }
            if seen.insert(node) {
                frontier.extend(
                    edges
                        .iter()
                        .filter_map(|&(from, to)| (from == node).then_some(to)),
                );
            }
        }
        false
    })
}
