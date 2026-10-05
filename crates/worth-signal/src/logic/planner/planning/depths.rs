//! Compact dependency-depth assignment for the discovered plan.
use crate::data::request_preparation::{self as preparation_budget, SignalPreparationBudget};
use std::collections::HashMap;

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::DedupedNodeBatch;

pub(super) struct DepthCache {
    index_by_node: HashMap<NodeId, usize>,
    depths: Vec<u32>,
}

impl DepthCache {
    pub(super) fn depth_for(&self, node: NodeId) -> Option<u32> {
        self.index_by_node
            .get(&node)
            .and_then(|index| self.depths.get(*index).copied())
    }

    pub(super) fn max_depth(&self) -> usize {
        self.depths.iter().copied().max().unwrap_or(0) as usize
    }
}

pub(super) fn compute_depths(
    graph: &mut SignalGraph,
    planned_nodes: &[NodeId],
    mut work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut SignalPreparationBudget>,
) -> Result<DepthCache, SignalError> {
    // hashbrown rounds buckets to a power of two and retains one control byte
    // per bucket. Four times the entries covers load factor and bucket rounding.
    preparation_budget::claim_vec::<(NodeId, usize)>(
        preparation.as_deref_mut(),
        planned_nodes.len().saturating_mul(4),
    )?;
    preparation_budget::claim_vec::<u8>(
        preparation.as_deref_mut(),
        planned_nodes.len().saturating_mul(4).saturating_add(16),
    )?;
    preparation_budget::claim_vec::<u32>(
        preparation.as_deref_mut(),
        planned_nodes.len().saturating_mul(2),
    )?;
    preparation_budget::claim_vec::<Vec<usize>>(preparation.as_deref_mut(), planned_nodes.len())?;
    preparation_budget::claim_vec::<NodeId>(preparation.as_deref_mut(), planned_nodes.len())?;
    let mut index_by_node = HashMap::with_capacity(planned_nodes.len());
    for (index, node) in planned_nodes.iter().copied().enumerate() {
        index_by_node.insert(node, index);
    }

    let mut indegree = vec![0_u32; planned_nodes.len()];
    let mut outgoing = vec![Vec::<usize>::new(); planned_nodes.len()];
    for (node_index, &node) in planned_nodes.iter().enumerate() {
        for source in super::required_inputs::required_input_sources(
            graph,
            node,
            work.as_deref_mut(),
            preparation.as_deref_mut(),
        )? {
            let Some(&source_index) = index_by_node.get(&source) else {
                continue;
            };
            indegree[node_index] += 1;
            preparation_budget::push(
                &mut outgoing[source_index],
                node_index,
                preparation.as_deref_mut(),
            )?;
        }
    }

    let mut frontier = planned_nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| (indegree[index] == 0).then_some(*node))
        .collect::<Vec<_>>();
    frontier = DedupedNodeBatch::canonicalize_unordered(frontier).into_vec();

    let mut depths = vec![0_u32; planned_nodes.len()];
    let mut visited = 0usize;
    while let Some(node) = frontier.pop() {
        let node_index = *index_by_node
            .get(&node)
            .ok_or_else(|| SignalError::internal("planned node missing compact depth index"))?;
        visited += 1;
        let depth = super::required_inputs::required_input_sources(
            graph,
            node,
            work.as_deref_mut(),
            preparation.as_deref_mut(),
        )?
        .into_iter()
        .filter_map(|source| {
            index_by_node
                .get(&source)
                .and_then(|source_index| depths.get(*source_index).copied())
        })
        .max()
        .map_or(0, |parent| parent + 1);
        depths[node_index] = depth;

        let mut newly_ready = Vec::new();
        for &child_index in &outgoing[node_index] {
            let degree = &mut indegree[child_index];
            *degree = degree.saturating_sub(1);
            if *degree == 0 {
                preparation_budget::push(
                    &mut newly_ready,
                    planned_nodes[child_index],
                    preparation.as_deref_mut(),
                )?;
            }
        }
        let newly_ready = DedupedNodeBatch::canonicalize_unordered(newly_ready).into_vec();
        for next in newly_ready.into_iter().rev() {
            preparation_budget::push(&mut frontier, next, preparation.as_deref_mut())?;
        }
    }

    if visited != planned_nodes.len() {
        return Err(SignalError::internal(
            "planner depth computation encountered a cycle in the planned graph",
        ));
    }

    Ok(DepthCache {
        index_by_node,
        depths,
    })
}
