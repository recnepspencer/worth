//! Read-only capacity admission before any checked evaluator is dispatched.
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::{CandidateEpochBasis, SignalGraph};
use crate::data::handle::NodeId;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    btree_structure_charge, RetainedStoragePreparation as Measurement,
    RetainedStoragePreparationDenial as Denial,
};
use crate::logic::planner::precompute::read_preparation::PrecomputeMapBasis;
use crate::logic::planner::EligibleTask;
use crate::logic::prepared::PreparedDependencyCapture;
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::{ExecutionResourceLease, MapKernelContext};

mod apply_capacity;
mod candidate_selection;
mod candidate_shape;
mod cause_capacity;
mod diagnostic_capacity;
mod grant;
mod lifecycle;
mod node_selection;
mod subscriber_sources;
mod units;
use crate::logic::planner::apply::workspace::ApplyMemberBasis;
use grant::ApplyMemberCapacity;
pub(in crate::logic::planner) use grant::{CheckedApplyCapacity, CheckedEpochAdmissionGrant};
use units::{bytes, overflow};

pub(super) struct ResourceAdmission<'lease, 'authority> {
    available: u64,
    fixed: u64,
    copies: u64,
    selected: BTreeSet<NodeId>,
    producers: BTreeSet<NodeId>,
    consumers: BTreeSet<NodeId>,
    consumer_causes: BTreeMap<NodeId, (usize, u64, usize)>,
    waiters: BTreeSet<NodeId>,
    sources: BTreeMap<NodeId, (usize, u64)>,
    owner_shape: u64,
    node_root_growth: u64,
    cause_transition_count: usize,
    width: usize,
    result_grant_bytes: u64,
    declared_result_minimum: u64,
    lease: worth_execution::ExecutionRequest<'lease, 'authority>,
    candidate_lease: Option<ExecutionResourceLease<'authority>>,
    apply_bases: Vec<ApplyMemberBasis>,
    candidate_bases: Vec<CandidateEpochBasis>,
    precompute_bases: Vec<PrecomputeMapBasis>,
}

impl<'lease, 'authority> ResourceAdmission<'lease, 'authority> {
    pub(super) fn consider(
        &mut self,
        graph: &SignalGraph,
        task: &EligibleTask,
        comparator: &impl ComparatorPolicyResolver,
        mut request: Option<&mut MapKernelContext<'_, '_>>,
        budget: &mut SignalPreparationBudget,
    ) -> Result<bool, SignalError> {
        let contract = graph.get_contract(task.node)?;
        let declaration = contract
            .execution
            .bounded_inputs
            .as_ref()
            .ok_or_else(|| SignalError::invalid_input("missing bounded inputs"))?;
        let declared_result_minimum = self.declared_result_minimum.max(
            contract
                .execution
                .max_checked_result_heap_bytes
                .unwrap_or(0),
        );
        let capture = PreparedDependencyCapture::checked_capture_heap_bound(declaration)
            .ok_or_else(overflow)?;
        let scope_heap = declaration
            .captured_scope_heap_bound()
            .ok_or_else(overflow)?;
        let fanout = graph.subscribers_of(task.node)?;
        let current = graph.current_runtime_dependencies_of(task.node)?;
        let mark = budget.checkpoint();
        let source_capacity = declaration.as_slice().len().saturating_add(current.len());
        if let Err(error) = budget.claim_vec::<NodeId>(source_capacity) {
            return if self.width == 0 {
                Err(error)
            } else {
                Ok(false)
            };
        }
        let new_waiters = match graph.epoch_waiter_closure(
            task.node,
            &self.waiters,
            request.as_deref_mut(),
            budget,
        ) {
            Ok(waiters) => waiters,
            Err(SignalError::PreparationMemoryExhausted { .. }) if self.width > 0 => {
                budget.release(mark);
                return Ok(false);
            }
            Err(error) => return Err(error),
        };
        let node_capacity = source_capacity
            .saturating_add(fanout.len())
            .saturating_add(new_waiters.len())
            .saturating_add(1);
        if let Err(error) = budget.claim_vec::<NodeId>(node_capacity) {
            budget.release(mark);
            return if self.width == 0 {
                Err(error)
            } else {
                Ok(false)
            };
        }
        super::super::work::checkpoint(
            request.as_deref_mut(),
            source_capacity
                .saturating_add(fanout.len())
                .saturating_add(new_waiters.len())
                .saturating_add(1),
        )?;
        let mut sources = Vec::with_capacity(source_capacity);
        sources.extend(declaration.as_slice().iter().map(|input| input.source));
        sources.extend(current.iter().map(|edge| edge.source()));
        sources.sort_unstable();
        sources.dedup();
        let mut nodes = Vec::with_capacity(node_capacity);
        nodes.push(task.node);
        nodes.extend_from_slice(&sources);
        nodes.extend_from_slice(fanout);
        nodes.extend(new_waiters.iter().copied());
        nodes.sort_unstable();
        nodes.dedup();

        let candidate_shape = candidate_shape::CandidateShape::measure(
            self,
            graph,
            task.node,
            &nodes,
            &sources,
            fanout,
            new_waiters.len(),
        )?;
        let index_scratch = candidate_shape.index_scratch()?;
        let candidate_shape::CandidateShape {
            next_width,
            owner_shape,
            owner_shape_growth,
            selected_index,
            producer_index,
            consumer_index,
            waiter_index,
            waiter_scratch,
            source_index,
            basis_growth_capacity,
            cause_transition_count,
        } = candidate_shape;
        let (subscriber_storage, next_source_bounds) =
            match subscriber_sources::measure(graph, &self.sources, &sources, budget) {
                Ok(measured) => measured,
                Err(SignalError::PreparationMemoryExhausted { .. }) if self.width > 0 => {
                    budget.release(mark);
                    return Ok(false);
                }
                Err(error) => return Err(error),
            };
        let resolved_policy = comparator.policy_for_node(
            task.node,
            graph.node_eval_config(task.node)?.comparator.as_ref(),
        );
        let mut measurement = Measurement::new(usize::MAX);
        let mut checkpoint = |visits: usize| {
            if let Some(request) = request.as_deref_mut() {
                request
                    .checkpoint(visits as u64)
                    .map_err(|stop| Denial::ExecutionStopped(stop.into()))?;
            }
            Ok(())
        };
        let old_edges_heap = apply_capacity::existing_edges_heap(
            current,
            &mut measurement.reborrow_with_checkpoint(&mut checkpoint),
        )?;
        let (apply_basis, cause_candidate, diagnostic_fixed, selected_heap, next_node_roots) = {
            let mut observed = measurement.reborrow_with_checkpoint(&mut checkpoint);
            let selected_heap =
                node_selection::clone_heap(self, graph, task.node, &nodes, &mut observed)?;
            let next_node_roots = match node_selection::root_growth(
                self,
                graph,
                task.node,
                &nodes,
                &mut observed,
                budget,
            ) {
                Ok(bound) => bound,
                Err(SignalError::PreparationMemoryExhausted { .. }) if self.width > 0 => {
                    budget.release(mark);
                    return Ok(false);
                }
                Err(error) => return Err(error),
            };
            let cause_candidate = cause_capacity::measure(
                graph,
                task.node,
                fanout,
                &self.consumer_causes,
                &mut observed,
                budget,
            )?;
            let diagnostic_fixed = diagnostic_capacity::measure(
                graph,
                task.node,
                scope_heap,
                old_edges_heap,
                current
                    .len()
                    .checked_add(declaration.as_slice().len())
                    .ok_or_else(overflow)?,
                &mut observed,
            )?;
            let apply_basis = apply_capacity::measure_member(
                graph,
                task.node,
                declaration,
                current,
                scope_heap,
                old_edges_heap,
                capture,
                resolved_policy,
                &mut observed,
            )?;
            (
                apply_basis,
                cause_candidate,
                diagnostic_fixed,
                selected_heap,
                next_node_roots,
            )
        };
        let node_root_growth = next_node_roots
            .checked_sub(self.node_root_growth)
            .ok_or_else(overflow)?;
        let candidate_basis = CandidateEpochBasis::measure_member(
            graph,
            task.node,
            request.as_deref_mut().ok_or_else(|| {
                SignalError::invalid_input("checked candidate basis needs request work")
            })?,
        )?;
        let candidate_preparation = candidate_basis.preparation_bytes().ok_or_else(overflow)?;
        let precompute_basis = PrecomputeMapBasis::new(capture, declaration.as_slice().len());
        let cause_growth = cause_candidate.growth;
        let cause_scope_index = btree_structure_charge::<NodeId, (usize, u64, usize)>(
            cause_candidate.new_scope_entries,
        )
        .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?
        .bytes();

        let metadata = candidate_selection::CandidateMetadata {
            edge_count: declaration.as_slice().len() as u64,
            scope_heap,
            capture,
            old_edges_heap,
            dependency_set: graph
                .epoch_dependency_set_capacity_bound(declaration.as_slice().len(), scope_heap)?,
            selected_heap,
            selected_index,
            producer_index,
            consumer_index,
            waiter_index,
            waiter_scratch,
            source_index,
            subscriber_storage,
            cause_growth,
            diagnostic_fixed,
            cause_scope_index,
            owner_shape_growth,
            node_root_growth,
            node_capacity,
            source_capacity,
            sources_len: sources.len(),
            fanout_len: fanout.len(),
            basis_growth_capacity,
            apply_basis: &apply_basis,
            candidate_preparation,
        }
        .fixed_bytes()?;
        let copies = candidate_selection::result_copies(fanout.len())?;
        let fixed = self.fixed.checked_add(metadata).ok_or_else(overflow)?;
        let all_copies = self.copies.checked_add(copies).ok_or_else(overflow)?;
        let grant = match candidate_selection::choose_grant(
            self.available,
            fixed,
            all_copies,
            next_width,
            declared_result_minimum,
            self.lease,
            self.candidate_lease.as_ref(),
            &self.apply_bases,
            &apply_basis,
            &self.candidate_bases,
            &candidate_basis,
            &self.precompute_bases,
            &precompute_basis,
            request,
        )? {
            Ok(grant) => grant,
            Err(denial) => {
                budget.release(mark);
                return self.decline_capacity(denial);
            }
        };
        if let Err(error) = budget.claim(index_scratch) {
            budget.release(mark);
            return if self.width == 0 {
                Err(error)
            } else {
                Ok(false)
            };
        }
        if basis_growth_capacity != 0 {
            if let Err(error) = budget.claim_vec::<ApplyMemberBasis>(basis_growth_capacity) {
                budget.release(mark);
                return if self.width == 0 {
                    Err(error)
                } else {
                    Ok(false)
                };
            }
            if let Err(error) = budget.claim_vec::<CandidateEpochBasis>(basis_growth_capacity) {
                budget.release(mark);
                return if self.width == 0 {
                    Err(error)
                } else {
                    Ok(false)
                };
            }
            if let Err(error) = budget.claim_vec::<PrecomputeMapBasis>(basis_growth_capacity) {
                budget.release(mark);
                return if self.width == 0 {
                    Err(error)
                } else {
                    Ok(false)
                };
            }
            self.apply_bases
                .reserve_exact(basis_growth_capacity - self.apply_bases.len());
            self.candidate_bases
                .reserve_exact(basis_growth_capacity - self.candidate_bases.len());
            self.precompute_bases
                .reserve_exact(basis_growth_capacity - self.precompute_bases.len());
        }
        if let Err(error) = budget.claim_vec::<ApplyMemberCapacity>(1) {
            budget.release(mark);
            return if self.width == 0 {
                Err(error)
            } else {
                Ok(false)
            };
        }
        if let Err(error) = budget.claim(cause_scope_index) {
            budget.release(mark);
            return if self.width == 0 {
                Err(error)
            } else {
                Ok(false)
            };
        }
        self.accept_selected(nodes, task.node, fanout);
        for scope in cause_candidate.next_scopes {
            self.consumer_causes.insert(
                scope.consumer,
                (scope.count, scope.heap, scope.scoped_count),
            );
        }
        self.waiters.extend(new_waiters);
        for (source, additions, bound) in next_source_bounds {
            self.sources.insert(source, (additions, bound));
        }
        self.fixed = fixed;
        self.copies = all_copies;
        self.owner_shape = owner_shape;
        self.node_root_growth = next_node_roots;
        self.cause_transition_count = cause_transition_count;
        self.width = next_width;
        self.result_grant_bytes = grant;
        self.declared_result_minimum = declared_result_minimum;
        self.apply_bases.push(apply_basis);
        self.candidate_bases.push(candidate_basis);
        self.precompute_bases.push(precompute_basis);
        Ok(true)
    }
}
