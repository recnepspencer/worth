use crate::data::request_preparation::{self as preparation_budget, SignalPreparationBudget};

use crate::data::bitset::DenseBitset;
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::proof::DedupedNodeBatch;
use crate::logic::evaluation::EvaluationRequestMode;

use super::super::types::{CandidateTask, MaybeStaleAdmission, TaskReason};
use super::admission::verify_required_context;
use super::unsettled_paths::{discover_unsettled_dependency_paths, UnsettledDependencyPaths};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PlannedNode {
    pub(super) direct_request: bool,
    pub(super) maybe_stale_admission: Option<MaybeStaleAdmission>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PlanningStats {
    pub(super) contract_pruned_count: u32,
}

pub(super) struct PlanTopology {
    pub(super) targets: Vec<NodeId>,
    pub(super) planned: Vec<Option<PlannedNode>>,
    pub(super) planned_nodes: Vec<NodeId>,
    pub(super) stats: PlanningStats,
}

pub(super) fn discover_plan_topology(
    graph: &mut SignalGraph,
    targets: &[NodeId],
    request_mode: EvaluationRequestMode,
    resolver: &mut impl ComparatorPolicyResolver,
    mut work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut SignalPreparationBudget>,
) -> Result<PlanTopology, SignalError> {
    let (arena, _, _, _) = graph.as_parts_mut();
    let arena_capacity = arena.len();
    preparation_budget::claim_vec::<Option<PlannedNode>>(
        preparation.as_deref_mut(),
        arena_capacity,
    )?;
    preparation_budget::claim_vec::<u64>(
        preparation.as_deref_mut(),
        arena_capacity.saturating_add(63) / 64,
    )?;
    preparation_budget::claim_vec::<NodeId>(preparation.as_deref_mut(), targets.len())?;
    let mut planned = vec![None::<PlannedNode>; arena_capacity];
    let mut planned_nodes = Vec::<NodeId>::new();
    let mut visiting = DenseBitset::new();
    let mut stats = PlanningStats::default();
    visiting.ensure_len(arena_capacity);
    let targets = DedupedNodeBatch::canonicalize_unordered(targets.iter().copied()).into_vec();
    let unsettled_paths = discover_unsettled_dependency_paths(
        graph,
        &targets,
        work.as_deref_mut(),
        preparation.as_deref_mut(),
    )?;

    for &target in &targets {
        graph.get_state(target)?;
        visit_node(
            graph,
            CandidateTask {
                node: target,
                request_mode,
                direct_request: true,
                trigger_reason: TaskReason::RequestedTarget,
            },
            resolver,
            &unsettled_paths,
            &mut visiting,
            &mut planned,
            &mut planned_nodes,
            &mut stats,
            work.as_deref_mut(),
            preparation.as_deref_mut(),
        )?;
    }

    Ok(PlanTopology {
        targets,
        planned,
        planned_nodes,
        stats,
    })
}

fn visit_node(
    graph: &mut SignalGraph,
    candidate: CandidateTask,
    resolver: &mut impl ComparatorPolicyResolver,
    unsettled_paths: &UnsettledDependencyPaths,
    visiting: &mut DenseBitset,
    planned: &mut [Option<PlannedNode>],
    planned_nodes: &mut Vec<NodeId>,
    stats: &mut PlanningStats,
    mut work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut SignalPreparationBudget>,
) -> Result<(), SignalError> {
    #[derive(Debug, Clone, Copy)]
    enum VisitFrame {
        Enter(CandidateTask),
        Exit(NodeId),
    }

    preparation_budget::claim_vec::<VisitFrame>(preparation.as_deref_mut(), 1)?;
    let mut stack = vec![VisitFrame::Enter(candidate)];
    while let Some(frame) = stack.pop() {
        super::super::precompute::work::checkpoint(work.as_deref_mut(), 1)?;
        match frame {
            VisitFrame::Enter(candidate) => {
                let node = candidate.node;
                let node_index = node.index() as usize;
                if visiting.contains(node_index) {
                    return Err(SignalError::invalid_input(format!(
                        "cycle detected while building evaluation plan at {node}"
                    )));
                }
                if let Some(existing) = &mut planned[node_index] {
                    existing.direct_request |= candidate.direct_request;
                    continue;
                }

                verify_required_context(
                    node,
                    graph.get_contract(node)?.semantics.required_context,
                )?;
                let state = graph.get_state(node)?;
                let should_include = unsettled_paths.contains(node)
                    || (candidate.direct_request
                        && matches!(candidate.request_mode, EvaluationRequestMode::ForceOnDemand));
                if !should_include {
                    stats.contract_pruned_count += 1;
                    continue;
                }

                planned[node_index] = Some(PlannedNode {
                    direct_request: candidate.direct_request,
                    maybe_stale_admission: None,
                });
                preparation_budget::push(planned_nodes, node, preparation.as_deref_mut())?;
                visiting.mark(node_index);
                preparation_budget::push(
                    &mut stack,
                    VisitFrame::Exit(node),
                    preparation.as_deref_mut(),
                )?;

                match state {
                    NodeState::Dirty => {
                        let dependencies = super::required_inputs::required_input_sources(
                            graph,
                            node,
                            work.as_deref_mut(),
                            preparation.as_deref_mut(),
                        )?;
                        for dependency in dependencies.into_iter().rev() {
                            preparation_budget::push(
                                &mut stack,
                                VisitFrame::Enter(CandidateTask {
                                    node: dependency,
                                    request_mode: candidate.request_mode,
                                    direct_request: false,
                                    trigger_reason: TaskReason::DependencyRequired,
                                }),
                                preparation.as_deref_mut(),
                            )?;
                        }
                    }
                    NodeState::MaybeStale => {
                        let preview = super::validation::preview_maybe_stale(
                            graph,
                            node,
                            resolver,
                            work.as_deref_mut(),
                            preparation.as_deref_mut(),
                        )?;
                        if let Some(existing) = &mut planned[node_index] {
                            existing.maybe_stale_admission = Some(MaybeStaleAdmission {
                                unchanged_at_admission: preview.unchanged,
                            });
                        }
                        let upstream_reason =
                            if matches!(candidate.trigger_reason, TaskReason::MaybeStaleValidation)
                            {
                                TaskReason::MaybeStaleValidation
                            } else {
                                TaskReason::DependencyRequired
                            };
                        let mut required_sources = preview.requires_upstream_evaluation;
                        let additional = super::required_inputs::required_input_sources(
                            graph,
                            node,
                            work.as_deref_mut(),
                            preparation.as_deref_mut(),
                        )?;
                        for source in additional {
                            preparation_budget::push(
                                &mut required_sources,
                                source,
                                preparation.as_deref_mut(),
                            )?;
                        }
                        super::super::precompute::work::checkpoint(
                            work.as_deref_mut(),
                            required_sources.len().saturating_mul(
                                required_sources.len().checked_ilog2().unwrap_or(0) as usize + 2,
                            ),
                        )?;
                        required_sources.sort_unstable();
                        required_sources.dedup();
                        for source in required_sources.into_iter().rev() {
                            preparation_budget::push(
                                &mut stack,
                                VisitFrame::Enter(CandidateTask {
                                    node: source,
                                    request_mode: candidate.request_mode,
                                    direct_request: false,
                                    trigger_reason: upstream_reason,
                                }),
                                preparation.as_deref_mut(),
                            )?;
                        }
                    }
                    NodeState::Clean
                        if candidate.direct_request
                            && matches!(
                                candidate.request_mode,
                                EvaluationRequestMode::ForceOnDemand
                            ) =>
                    {
                        let dependencies = super::required_inputs::required_input_sources(
                            graph,
                            node,
                            work.as_deref_mut(),
                            preparation.as_deref_mut(),
                        )?;
                        for dependency in dependencies.into_iter().rev() {
                            if !matches!(graph.get_state(dependency)?, NodeState::Clean) {
                                preparation_budget::push(
                                    &mut stack,
                                    VisitFrame::Enter(CandidateTask {
                                        node: dependency,
                                        request_mode: candidate.request_mode,
                                        direct_request: false,
                                        trigger_reason: TaskReason::DependencyRequired,
                                    }),
                                    preparation.as_deref_mut(),
                                )?;
                            }
                        }
                    }
                    NodeState::Clean => {
                        let dependencies = super::required_inputs::required_input_sources(
                            graph,
                            node,
                            work.as_deref_mut(),
                            preparation.as_deref_mut(),
                        )?;
                        for dependency in dependencies.into_iter().rev() {
                            if unsettled_paths.contains(dependency) {
                                preparation_budget::push(
                                    &mut stack,
                                    VisitFrame::Enter(CandidateTask {
                                        node: dependency,
                                        request_mode: candidate.request_mode,
                                        direct_request: false,
                                        trigger_reason: TaskReason::DependencyRequired,
                                    }),
                                    preparation.as_deref_mut(),
                                )?;
                            }
                        }
                    }
                }
            }
            VisitFrame::Exit(node) => {
                visiting.clear(node.index() as usize);
            }
        }
    }

    Ok(())
}
