mod dependency_edges;
mod snapshots;
mod subscribers;

use std::collections::HashMap;
use std::hash::Hash;

use crate::data::handle::NodeId;
use crate::data::node::NodeEntry;

use super::super::runtime::graph::{EdgeTopology, SignalGraph};
use super::schedule::CompactionFamily;

impl SignalGraph {
    #[cfg(test)]
    pub(crate) fn compact_graph_storage(&mut self) {
        self.compact_storage_family(CompactionFamily::DependencyEdges);
        self.compact_storage_family(CompactionFamily::Subscribers);
        self.compact_storage_family(CompactionFamily::Snapshots);
    }

    pub(in crate::data::graph) fn compact_storage_family(&mut self, family: CompactionFamily) {
        match family {
            CompactionFamily::DependencyEdges => self.compact_dependency_edge_storage(),
            CompactionFamily::Subscribers => self.compact_subscriber_edge_storage(),
            CompactionFamily::Snapshots => self.compact_dependency_snapshot_storage(),
        }
    }
}

impl EdgeTopology {
    pub(in crate::data::graph) fn prune_dead_dependency_edges(
        graph: &mut SignalGraph,
        node: NodeId,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<(), crate::data::error::SignalError> {
        if graph.arena.compaction.tombstone_count == 0 {
            return Ok(());
        }
        Self::prune_dependency_edges(graph, node, work)
    }

    pub(in crate::data::graph) fn prune_dead_subscriber_edges(
        graph: &mut SignalGraph,
        node: NodeId,
    ) -> Result<(), crate::data::error::SignalError> {
        if graph.arena.compaction.tombstone_count == 0 {
            return Ok(());
        }
        Self::prune_subscriber_edges(graph, node)
    }

    fn prune_dependency_edges(
        graph: &mut SignalGraph,
        node: NodeId,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<(), crate::data::error::SignalError> {
        work.reserve(graph.raw_dependencies_of(node)?.len().checked_mul(8))?;
        let has_stale = {
            let current = graph.raw_dependencies_of(node)?;
            current.iter().any(|edge| !graph.is_alive(edge.source()))
        };
        if has_stale {
            let updated = graph
                .raw_dependencies_of(node)?
                .iter()
                .filter(|edge| graph.is_alive(edge.source()))
                .cloned()
                .collect::<Vec<_>>();
            graph.set_dependency_edges_sorted_with_work(node, &updated, work)?;
        }
        Ok(())
    }

    fn prune_subscriber_edges(
        graph: &mut SignalGraph,
        node: NodeId,
    ) -> Result<(), crate::data::error::SignalError> {
        let has_stale = {
            let current = graph.raw_subscribers_of(node)?;
            current
                .iter()
                .any(|subscriber| !graph.is_alive(*subscriber))
        };
        if has_stale {
            let updated = graph
                .raw_subscribers_of(node)?
                .iter()
                .copied()
                .filter(|subscriber| graph.is_alive(*subscriber))
                .collect::<Vec<_>>();
            graph.set_subscribers_sorted(node, &updated)?;
        }
        Ok(())
    }
}

fn remap_live_entry_handles<OldId, NewId>(
    graph: &mut SignalGraph,
    mut read_id: impl FnMut(&NodeEntry) -> OldId,
    mut remap_id: impl FnMut(OldId) -> NewId,
    mut write_id: impl FnMut(&mut NodeEntry, NewId),
) where
    OldId: Eq + Hash + Copy,
    NewId: Copy,
{
    let mut id_map = HashMap::new();

    for index in 0..graph.arena.nodes.len() {
        let Some(node) = graph.live_node_id_at(index) else {
            continue;
        };
        let old_id = match graph.get_entry(node) {
            Ok(entry) => read_id(&entry),
            Err(_) => continue,
        };
        let new_id = *id_map.entry(old_id).or_insert_with(|| remap_id(old_id));
        if let Ok(mut live_entry) = graph.get_entry_mut(node) {
            write_id(&mut live_entry, new_id);
        }
    }
}
