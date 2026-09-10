use super::{DependencyEdge, NodeId, SignalError, SignalGraph};
use std::collections::BTreeSet;

/// Scratch grows with admitted edge visits, never unrelated arena population.
/// Collection capacity and tree overhead mean this is not an exact byte bound.
struct UpstreamTraversal<'graph> {
    visited: BTreeSet<NodeId>,
    stack: Vec<std::iter::Rev<std::slice::Iter<'graph, DependencyEdge>>>,
    visits: usize,
    maximum_visits: usize,
}

impl<'graph> UpstreamTraversal<'graph> {
    fn new(dependencies: &'graph [DependencyEdge], maximum_visits: usize) -> Self {
        Self {
            visited: BTreeSet::new(),
            stack: vec![dependencies.iter().rev()],
            visits: 0,
            maximum_visits,
        }
    }

    fn has_unsettled(
        &mut self,
        graph: &'graph SignalGraph,
        debit: &mut dyn FnMut(Option<usize>) -> Result<(), SignalError>,
    ) -> Result<bool, SignalError> {
        while let Some(edges) = self.stack.last_mut() {
            let Some(edge) = edges.next() else {
                self.stack.pop();
                continue;
            };
            if self.visits == self.maximum_visits {
                return Err(SignalError::UpstreamDependencyWorkExhausted {
                    maximum_visits: self.maximum_visits,
                });
            }
            self.visits += 1;
            let node = edge.source();
            // Bound search and structural moves by four visits per stored key
            // plus the new key, before insertion/allocation. Duplicate edges
            // still pay; deduplication cannot refund attempted work.
            debit(
                self.visited
                    .len()
                    .checked_add(1)
                    .and_then(|keys| keys.checked_mul(4)),
            )?;
            if !self.visited.insert(node) {
                continue;
            }
            graph.invalidation_performed_counter_state().add(
                crate::data::telemetry::InvalidationPerformedCounter::NonSemanticNodeVisits,
                1,
            );
            if graph.get_state(node)? != crate::data::node::NodeState::Clean {
                return Ok(true);
            }
            let dependencies = graph.current_runtime_dependencies_of(node)?;
            if !dependencies.is_empty() {
                // A growing vector may move every existing frame.
                debit(if self.stack.len() == self.stack.capacity() {
                    self.stack.len().checked_add(1)
                } else {
                    Some(1)
                })?;
                self.stack.push(dependencies.iter().rev());
            }
        }
        Ok(false)
    }
}

impl SignalGraph {
    pub(crate) fn has_current_unsettled_upstream(
        &self,
        target: NodeId,
    ) -> Result<bool, SignalError> {
        let dependencies = self.current_runtime_dependencies_of(target)?;
        if dependencies.is_empty() {
            return Ok(false);
        }
        UpstreamTraversal::new(
            dependencies,
            self.installed_runtime_policy()
                .maximum_upstream_dependency_visits(),
        )
        .has_unsettled(self, &mut |_| Ok(()))
    }

    pub(crate) fn conditional_has_current_unsettled_upstream(
        &self,
        target: NodeId,
        work: &mut crate::data::retained_storage::RetainedStoragePreparation,
    ) -> Result<bool, SignalError> {
        let dependencies = self.current_runtime_dependencies_of(target)?;
        if dependencies.is_empty() {
            return Ok(false);
        }
        crate::data::conditional_execution::conditional_work::reserve(work, Some(1))?;
        UpstreamTraversal::new(
            dependencies,
            self.installed_runtime_policy()
                .maximum_upstream_dependency_visits(),
        )
        .has_unsettled(self, &mut |visits| {
            crate::data::conditional_execution::conditional_work::reserve(work, visits)
        })
    }
}

#[cfg(test)]
mod tests;
