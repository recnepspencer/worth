use std::collections::BTreeMap;
use worth_execution::MapKernelContext;

use crate::data::dependency::{CanonicalDependencies, DependencyEdge};
use crate::data::error::SignalError;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;

pub(super) fn canonicalize_and_preflight(
    graph: &SignalGraph,
    reconciliations: &[(NodeId, &[DependencyEdge])],
) -> Result<Vec<(NodeId, CanonicalDependencies)>, SignalError> {
    canonicalize_and_preflight_with_work(graph, reconciliations, None)
}

pub(super) fn canonicalize_and_preflight_with_work(
    graph: &SignalGraph,
    reconciliations: &[(NodeId, &[DependencyEdge])],
    mut work: Option<&mut MapKernelContext<'_, '_>>,
) -> Result<Vec<(NodeId, CanonicalDependencies)>, SignalError> {
    let mut desired_by_node = BTreeMap::new();
    for &(node, desired) in reconciliations {
        checkpoint(work.as_deref_mut(), desired.len().saturating_add(1))?;
        graph.validate_handle(node)?;
        if desired_by_node.contains_key(&node) {
            return Err(SignalError::invalid_input(format!(
                "dependency mutation contains duplicate target {node}"
            )));
        }
        for edge in desired {
            graph.validate_handle(edge.source())?;
        }
        desired_by_node.insert(node, CanonicalDependencies::new(desired.iter().cloned()));
    }

    let mut changed = Vec::new();
    for (&node, desired) in &desired_by_node {
        checkpoint(work.as_deref_mut(), 1)?;
        if graph.raw_dependencies_of(node)? != desired.as_slice() {
            changed.push(node);
        }
    }
    reject_batch_cycles(graph, &changed, &desired_by_node, work)?;
    Ok(desired_by_node.into_iter().collect())
}

fn reject_batch_cycles(
    graph: &SignalGraph,
    roots: &[NodeId],
    desired_by_node: &BTreeMap<NodeId, CanonicalDependencies>,
    mut work: Option<&mut MapKernelContext<'_, '_>>,
) -> Result<(), SignalError> {
    let capacity = graph.arena_capacity();
    checkpoint(work.as_deref_mut(), capacity.saturating_mul(2))?;
    let mut visit_state = vec![0_u8; capacity];
    let mut active_positions = vec![None::<usize>; capacity];
    for &root in roots {
        checkpoint(work.as_deref_mut(), 1)?;
        if visit_state[root.index() as usize] == 2 {
            continue;
        }
        let root_sources = dependency_sources(graph, root, desired_by_node, work.as_deref_mut())?;
        let mut stack = vec![DependencyFrame::new(root, root_sources)];
        visit_state[root.index() as usize] = 1;
        active_positions[root.index() as usize] = Some(0);
        while let Some(frame) = stack.last_mut() {
            checkpoint(work.as_deref_mut(), 1)?;
            let Some(next) = frame.next_source() else {
                let completed = stack.pop().expect("active dependency frame must exist");
                let index = completed.node.index() as usize;
                active_positions[index] = None;
                visit_state[index] = 2;
                continue;
            };
            let next_index = next.index() as usize;
            if let Some(cycle_start) = active_positions[next_index] {
                checkpoint(work.as_deref_mut(), stack.len().saturating_add(1))?;
                let mut path = stack[cycle_start..]
                    .iter()
                    .map(|frame| frame.node)
                    .collect::<Vec<_>>();
                path.push(next);
                return Err(SignalError::cycle_detected(path));
            }
            if visit_state[next_index] == 2 {
                continue;
            }
            let next_position = stack.len();
            let next_sources =
                dependency_sources(graph, next, desired_by_node, work.as_deref_mut())?;
            stack.push(DependencyFrame::new(next, next_sources));
            visit_state[next_index] = 1;
            active_positions[next_index] = Some(next_position);
        }
    }
    Ok(())
}

fn dependency_sources(
    graph: &SignalGraph,
    node: NodeId,
    desired_by_node: &BTreeMap<NodeId, CanonicalDependencies>,
    work: Option<&mut MapKernelContext<'_, '_>>,
) -> Result<Vec<NodeId>, SignalError> {
    let edges = match desired_by_node.get(&node) {
        Some(desired) => desired.as_slice(),
        None => graph.raw_dependencies_of(node)?,
    };
    checkpoint(work, edges.len())?;
    Ok(edges
        .iter()
        .map(DependencyEdge::source)
        .filter(|source| graph.is_alive(*source))
        .collect())
}

fn checkpoint(
    work: Option<&mut MapKernelContext<'_, '_>>,
    units: usize,
) -> Result<(), SignalError> {
    if let Some(work) = work {
        let units = u64::try_from(units)
            .map_err(|_| SignalError::invalid_input("dependency preflight work overflow"))?;
        work.checkpoint(units)
            .map_err(SignalError::execution_checkpoint_stopped)?;
    }
    Ok(())
}

struct DependencyFrame {
    node: NodeId,
    sources: Vec<NodeId>,
    next_index: usize,
}

impl DependencyFrame {
    fn new(node: NodeId, sources: Vec<NodeId>) -> Self {
        Self {
            node,
            sources,
            next_index: 0,
        }
    }

    fn next_source(&mut self) -> Option<NodeId> {
        let source = self.sources.get(self.next_index).copied();
        self.next_index += usize::from(source.is_some());
        source
    }
}
