//! Independent physical multigraph model with distinct transitive reachability.
pub use super::{installation::*, outcome::*, read, schema::*};
pub use std::collections::{BTreeMap, BTreeSet};
pub use worth_query_host::facade::{
    application_invariants::EntityReference, primary_graph::WorthQueryRelationLawDenial,
};
pub type Bag = BTreeMap<(u64, u64), usize>;
pub fn reachable(graph: &Bag, source: u64) -> usize {
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
