use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::CheckpointNodeImage;
use crate::data::proof::DedupedNodeBatch;
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone)]
struct NodePatch {
    original: CheckpointNodeImage,
    original_dependency_sources: Vec<NodeId>,
    original_branch_mutation: crate::data::graph::signal_graph::BranchMutationNodeImage,
}

/// Sparse patch storage with O(touched) rollback/clear semantics.
#[derive(Debug, Clone, Default)]
pub(super) struct SparsePatchBuffer {
    patches: Vec<(usize, NodePatch)>,
    index_by_node: HashMap<usize, usize>,
}

impl SparsePatchBuffer {
    pub(super) fn new() -> Self {
        Self {
            patches: Vec::new(),
            index_by_node: HashMap::new(),
        }
    }

    pub(super) fn stage_original(
        &mut self,
        graph: &SignalGraph,
        node: NodeId,
    ) -> Result<(), SignalError> {
        let index = node.index() as usize;
        if !self.index_by_node.contains_key(&index) {
            let original = graph.node_checkpoint_image(node)?;
            let original_dependency_sources = graph.dependency_sources_of(node)?;
            let original_branch_mutation = graph.branch_mutation_node_image(node);
            self.index_by_node.insert(index, self.patches.len());
            self.patches.push((
                index,
                NodePatch {
                    original,
                    original_dependency_sources,
                    original_branch_mutation,
                },
            ));
        }
        Ok(())
    }

    pub(super) fn touched_count(&self) -> usize {
        self.patches.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.patches.is_empty()
    }

    pub(super) fn touched_nodes(&self, graph: &SignalGraph) -> DedupedNodeBatch {
        DedupedNodeBatch::canonicalize_unordered(
            self.patches
                .iter()
                .filter_map(|(index, _)| graph.live_node_id_at(*index))
                .collect::<Vec<_>>(),
        )
    }

    /// Commit path: graph already contains staged changes, so clear patches only.
    pub(super) fn commit_and_clear(&mut self) {
        self.patches.clear();
        self.index_by_node.clear();
    }

    /// Roll back graph to staged originals, then clear patch set.
    #[cfg(test)]
    pub(super) fn rollback_and_clear(
        &mut self,
        graph: &mut SignalGraph,
    ) -> Result<(), SignalError> {
        std::mem::take(self).rollback_from_packet(graph)
    }

    pub(super) fn collect_dependency_sources_for_rollback(
        &self,
        graph: &SignalGraph,
    ) -> Result<Vec<NodeId>, SignalError> {
        let mut sources = BTreeSet::new();

        for (index, patch) in &self.patches {
            let node = graph
                .live_node_id_at(*index)
                .ok_or_else(|| SignalError::internal("rollback encountered stale patch node"))?;
            for &source in &patch.original_dependency_sources {
                sources.insert(source);
            }
            for source in graph.dependency_sources_of(node)? {
                sources.insert(source);
            }
        }

        Ok(sources.into_iter().collect())
    }

    pub(super) fn rollback_from_packet(
        mut self,
        graph: &mut SignalGraph,
    ) -> Result<(), SignalError> {
        #[cfg(debug_assertions)]
        let mut touched = graph.topology_debug_asserts_enabled().then(Vec::new);
        self.patches.sort_by_key(|(index, _)| *index);
        for (index, patch) in self.patches {
            let node = graph
                .live_node_id_at(index)
                .ok_or_else(|| SignalError::internal("rollback encountered stale patch node"))?;
            let current_sources = graph.dependency_sources_of(node)?;
            graph.reconcile_subscriber_sets(
                node,
                &current_sources,
                &patch.original_dependency_sources,
            )?;
            graph.replace_entry_from_checkpoint_image(node, patch.original)?;
            graph.restore_branch_mutation_node_image(node, patch.original_branch_mutation);
            #[cfg(debug_assertions)]
            if let Some(touched) = &mut touched {
                touched.push((node, current_sources, patch.original_dependency_sources));
            }
        }

        #[cfg(debug_assertions)]
        if let Some(touched) = touched {
            for (node, former_sources, desired_sources) in touched {
                graph.debug_assert_bidirectional_consistency_at(
                    node,
                    &former_sources,
                    &desired_sources,
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SparsePatchBuffer;
    use crate::data::dependency::DependencySnapshot;
    use crate::data::error::SignalError;
    use crate::data::graph::SignalGraph;
    use crate::data::node::NodeState;
    use crate::logic::invalidation::mark_dirty;
    use crate::tests::support::*;

    #[test]
    fn rollback_clears_only_touched_entries_and_preserves_untouched() -> Result<(), SignalError> {
        let mut graph = SignalGraph::new();
        let a = graph.create_node();
        let b = graph.create_node();
        graph.append_dependency(b, a, ASPECT_B)?;

        let before_a = graph.get_state(a)?;
        let before_b = graph.get_state(b)?;

        let mut patches = SparsePatchBuffer::new();
        patches.stage_original(&graph, a)?;
        patches.stage_original(&graph, b)?;
        assert_eq!(patches.touched_count(), 2);

        mark_dirty(&mut graph, a, ASPECT_B)?;
        assert_eq!(graph.get_state(a)?, NodeState::Dirty);

        patches.rollback_and_clear(&mut graph)?;
        assert_eq!(graph.get_state(a)?, before_a);
        assert_eq!(graph.get_state(b)?, before_b);
        assert_eq!(patches.touched_count(), 0);
        Ok(())
    }

    #[test]
    fn stage_high_index_node_grows_capacity_safely() -> Result<(), SignalError> {
        let mut graph = SignalGraph::new();
        let mut target = graph.create_node();
        for _ in 0..10_000 {
            target = graph.create_node();
        }

        let mut patches = SparsePatchBuffer::new();
        patches.stage_original(&graph, target)?;
        assert_eq!(patches.touched_count(), 1);
        Ok(())
    }

    #[test]
    fn rollback_then_restage_has_no_ghost_data() -> Result<(), SignalError> {
        let mut graph = SignalGraph::new();
        let a = graph.create_node();
        let baseline = graph.get_entry(a)?.clone();
        let mut patches = SparsePatchBuffer::new();

        patches.stage_original(&graph, a)?;
        graph
            .get_entry_mut(a)?
            .set_aspect_version(version_ab(0, 10));
        patches.rollback_and_clear(&mut graph)?;
        assert_eq!(*graph.get_entry(a)?, baseline);
        assert_eq!(patches.touched_count(), 0);

        patches.stage_original(&graph, a)?;
        graph
            .get_entry_mut(a)?
            .set_aspect_version(version_ab(0, 11));
        patches.commit_and_clear();
        assert_eq!(graph.get_entry(a)?.get_aspect_version(), version_ab(0, 11));
        assert_eq!(patches.touched_count(), 0);
        Ok(())
    }

    #[test]
    fn rollback_restores_dependency_snapshot_handle() -> Result<(), SignalError> {
        let mut graph = SignalGraph::new();
        let a = graph.create_node();
        let b = graph.create_node();
        graph.append_dependency(b, a, ASPECT_B)?;

        let mut baseline = DependencySnapshot::empty();
        baseline.record(a, ASPECT_B, 3, None);
        graph.set_dep_snapshot(b, baseline.clone())?;

        let mut patches = SparsePatchBuffer::new();
        patches.stage_original(&graph, b)?;

        let mut updated = DependencySnapshot::empty();
        updated.record(a, ASPECT_B, 7, None);
        graph.set_dep_snapshot(b, updated)?;

        patches.rollback_and_clear(&mut graph)?;
        assert_eq!(graph.get_dep_snapshot(b)?, &baseline);
        Ok(())
    }
}

#[cfg(all(test, debug_assertions))]
mod topology_check_tests;
