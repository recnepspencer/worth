use crate::data::aspect::AspectMask;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::request_preparation::{self as preparation_budget, SignalPreparationBudget};

pub(super) struct UnsettledDependencyPaths {
    required: Vec<bool>,
}

impl UnsettledDependencyPaths {
    pub(super) fn contains(&self, node: NodeId) -> bool {
        self.required
            .get(node.index() as usize)
            .copied()
            .unwrap_or(false)
    }
}

pub(super) fn discover_unsettled_dependency_paths(
    graph: &mut SignalGraph,
    targets: &[NodeId],
    mut work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut SignalPreparationBudget>,
) -> Result<UnsettledDependencyPaths, SignalError> {
    #[derive(Clone, Copy)]
    enum Frame {
        Enter(NodeId),
        Exit(NodeId),
    }

    let capacity = graph.arena_capacity();
    super::super::precompute::work::checkpoint(
        work.as_deref_mut(),
        capacity.saturating_mul(2).saturating_add(targets.len()),
    )?;
    preparation_budget::claim_vec::<u8>(preparation.as_deref_mut(), capacity.saturating_mul(2))?;
    let mut visit_state = vec![0_u8; capacity];
    let mut required = vec![false; capacity];
    for &target in targets {
        preparation_budget::claim_vec::<Frame>(preparation.as_deref_mut(), 1)?;
        let mut stack = vec![Frame::Enter(target)];
        while let Some(frame) = stack.pop() {
            super::super::precompute::work::checkpoint(work.as_deref_mut(), 1)?;
            let node = match frame {
                Frame::Enter(node) | Frame::Exit(node) => node,
            };
            let index = node.index() as usize;
            match frame {
                Frame::Enter(_) => match visit_state.get(index).copied() {
                    Some(2) => continue,
                    Some(1) => {
                        return Err(SignalError::invalid_input(format!(
                            "cycle detected while discovering unsettled dependency paths at {node}"
                        )))
                    }
                    Some(0) => {}
                    _ => {
                        return Err(SignalError::invalid_input(format!(
                            "dependency path references unavailable node {node}"
                        )))
                    }
                },
                Frame::Exit(_) => {
                    let state = graph.get_state(node)?;
                    let dependency_count = graph.current_runtime_dependencies_of(node)?.len();
                    super::super::precompute::work::checkpoint(
                        work.as_deref_mut(),
                        dependency_count.saturating_add(1),
                    )?;
                    if work.is_none() {
                        graph.refresh_runtime_dependencies_of(node)?;
                    }
                    let contract = graph.get_contract(node)?;
                    let dependency_requires_work = graph
                        .current_runtime_dependencies_of(node)?
                        .iter()
                        .any(|edge| {
                            let scopes = edge.scope_ref().map(std::slice::from_ref).unwrap_or(&[]);
                            required[edge.source().index() as usize]
                                && contract.cares_about_change(
                                    AspectMask::from_aspect(edge.aspect()),
                                    scopes,
                                )
                        });
                    required[index] =
                        !matches!(state, NodeState::Clean) || dependency_requires_work;
                    visit_state[index] = 2;
                    continue;
                }
            }
            graph.get_state(node)?;
            visit_state[index] = 1;
            preparation_budget::push(&mut stack, Frame::Exit(node), preparation.as_deref_mut())?;
            let sources = if !matches!(graph.get_state(node)?, NodeState::Clean) {
                super::required_inputs::required_input_sources(
                    graph,
                    node,
                    work.as_deref_mut(),
                    preparation.as_deref_mut(),
                )?
            } else {
                let dependency_count = graph.current_runtime_dependencies_of(node)?.len();
                super::super::precompute::work::checkpoint(
                    work.as_deref_mut(),
                    dependency_count.saturating_add(1),
                )?;
                if work.is_none() {
                    graph.refresh_runtime_dependencies_of(node)?;
                }
                preparation_budget::claim_vec::<NodeId>(
                    preparation.as_deref_mut(),
                    dependency_count,
                )?;
                graph
                    .current_runtime_dependencies_of(node)?
                    .iter()
                    .map(|edge| edge.source())
                    .collect()
            };
            for source in sources.into_iter().rev() {
                preparation_budget::push(
                    &mut stack,
                    Frame::Enter(source),
                    preparation.as_deref_mut(),
                )?;
            }
        }
    }
    Ok(UnsettledDependencyPaths { required })
}
