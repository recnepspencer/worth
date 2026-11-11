//! Declaration cycles are checked against declarations, independently of discovered edges.
use crate::data::dependency::CanonicalDependencies;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use std::collections::{BTreeMap, BTreeSet};

impl SignalGraph {
    pub(in crate::data::graph) fn validate_declared_edits(
        &self,
        edits: &[(NodeId, CanonicalDependencies)],
    ) -> Result<(), SignalError> {
        let mut proposed = BTreeMap::new();
        for (node, edges) in edits {
            self.validate_handle(*node)?;
            if proposed.insert(*node, edges.as_slice()).is_some() {
                return Err(SignalError::invalid_input(
                    "duplicate authored dependency target",
                ));
            }
            for edge in edges.as_slice() {
                self.validate_handle(edge.source())?;
            }
        }
        let mut complete = BTreeSet::new();
        let mut active = BTreeMap::new();
        for (root, _) in edits {
            let mut stack = vec![(*root, 0usize)];
            active.insert(*root, 0usize);
            while let Some((node, index)) = stack.last_mut() {
                let edges = match proposed.get(node) {
                    Some(edges) => *edges,
                    None => self.declared_dependencies_of(*node)?,
                };
                let Some(edge) = edges.get(*index) else {
                    complete.insert(*node);
                    active.remove(node);
                    stack.pop();
                    continue;
                };
                *index += 1;
                let source = edge.source();
                if let Some(start) = active.get(&source).copied() {
                    let mut path = stack[start..]
                        .iter()
                        .map(|(node, _)| *node)
                        .collect::<Vec<_>>();
                    path.push(source);
                    return Err(SignalError::cycle_detected(path));
                }
                if !complete.contains(&source) {
                    active.insert(source, stack.len());
                    stack.push((source, 0));
                }
            }
        }
        Ok(())
    }
}
