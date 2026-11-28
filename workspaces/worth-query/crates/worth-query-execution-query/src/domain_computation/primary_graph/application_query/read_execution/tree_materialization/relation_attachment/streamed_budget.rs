//! Batched streaming relation attachment under an active `streaming_window_bytes`
//! budget, charging traversal preparation work, iterator memory, candidate
//! predicate buffers, selected child targets, and retained-target sort work
//! incrementally via `try_visit_adjacency_ids` without ever collecting full
//! adjacency for a high-fan-out root.

use worth_execution::{ExecutionWorkCeiling, MapKernelStop, WorkCeilingDenial};
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_declaration::facade::application_query::ApplicationQueryResultTraversalDirection;
use worth_query_installation::facade::{
    WorthQueryInstalledGraphReadContract, WorthQueryInstalledGraphRelation,
};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::runtime::{
    ProjectionAspectScope, RelationalAdjacencyDirection, RelationalAdjacencyVisit,
    VisibilityProjectionView,
};
use worth_relational::facade::storage::RecordLifecycleState;
use worth_relational::facade::transactions::RelationalExecutionDenialCause;

use super::super::relation_distribution::distribute_relation_rows;
use super::super::relation_occurrence::{attach_occurrences, RelationProjectionTarget};
use super::super::relation_target_filter::retain_matching_targets;
use super::super::{
    project_nodes, ActiveResultTreeCollectionSelection, ResultTreeWork,
    WorthQueryApplicationProjectionNode, WorthQueryApplicationReadExecutionDenial,
};
use super::validate_relation_cardinality;
use crate::domain_computation::primary_graph::application_query::read_execution::denial::{
    read_execution_denial, relational_read_denial, WorthQueryApplicationReadExecutionDenialKind,
};

const ADJACENCY_BATCH_SIZE: usize = 32;

fn sort_work_units(len: usize) -> Option<u64> {
    if len <= 1 {
        return Some(0);
    }
    let log2 = u64::from(usize::BITS - (len - 1).leading_zeros());
    u64::try_from(len).ok()?.checked_mul(log2)
}

fn map_kernel_stop(stop: MapKernelStop, subject: &str) -> WorthQueryApplicationReadExecutionDenial {
    match stop {
        MapKernelStop::Cancelled => read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::Cancelled,
            subject,
        ),
        MapKernelStop::DeadlineElapsed => read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::DeadlineExceeded,
            subject,
        ),
        MapKernelStop::WorkCounterOverflow => {
            relational_read_denial(RelationalExecutionDenialCause::WorkCounterOverflow, subject)
        }
        _ => read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
            subject,
        ),
    }
}

fn debit_execution_work(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    cumulative_work: &mut u64,
    units: u64,
    subject: &str,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    if units == 0 {
        return Ok(());
    }
    *cumulative_work = cumulative_work.checked_add(units).ok_or_else(|| {
        relational_read_denial(RelationalExecutionDenialCause::WorkCounterOverflow, subject)
    })?;
    if let Some(ceiling) = execution.work_ceiling()
        && *cumulative_work > ceiling
    {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
            subject,
        ));
    }
    let step = execution.in_scope(|lease| {
        ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
            lease,
            None,
            |context| context.checkpoint(units),
        )
    });
    match step {
        Ok(Ok((Ok(()), _))) => Ok(()),
        Ok(Ok((Err(stop), _)))
        | Ok(Err(WorkCeilingDenial::Stopped(stop)))
        | Err(WorkCeilingDenial::Stopped(stop)) => Err(map_kernel_stop(stop, subject)),
        _ => Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
            subject,
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn attach_relation_within_window_budget(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    read: &crate::domain_computation::primary_graph::application_query::read_execution::tree_materialization::read_basis::ResultTreeRead<'_>,
    projection: &VisibilityProjectionView,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    parameters: &WorthQueryAdmittedApplicationQueryParameters,
    relation: &WorthQueryInstalledGraphRelation,
    relation_kind: KindId,
    parents: &mut [WorthQueryApplicationProjectionNode],
    work: &mut ResultTreeWork,
    collection_selection: &mut ActiveResultTreeCollectionSelection,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let adj_dir = match relation.direction() {
        ApplicationQueryResultTraversalDirection::Forward => RelationalAdjacencyDirection::Outgoing,
        ApplicationQueryResultTraversalDirection::Reverse => RelationalAdjacencyDirection::Incoming,
    };
    let mut counts = Vec::with_capacity(parents.len());
    let mut predicate_counts = Vec::with_capacity(parents.len());
    let mut predicate_sources = Vec::new();
    let mut selected_targets = Vec::new();
    let mut cumulative_exec_work = 0u64;

    for parent in parents.iter() {
        let parent_id = parent.entity_id();
        work.checkpoint(relation.result_path())?;
        let mut batch = Vec::with_capacity(ADJACENCY_BATCH_SIZE);
        let mut parent_targets = Vec::new();
        let mut parent_predicate_count = 0usize;
        let mut iterator_bytes = 0u64;

        let visit_res = projection
            .try_visit_adjacency_ids(execution, parent_id, relation_kind, adj_dir, |visit| {
                match visit {
                    RelationalAdjacencyVisit::List => {
                        debit_execution_work(
                            execution,
                            &mut cumulative_exec_work,
                            1,
                            relation.result_path(),
                        )?;
                        work.charge_adjacency(1, 0, 0, relation.result_path())?;
                    }
                    RelationalAdjacencyVisit::Prepare {
                        work: prep_work,
                        bytes: prep_bytes,
                    } => {
                        if prep_work > 0 {
                            debit_execution_work(
                                execution,
                                &mut cumulative_exec_work,
                                prep_work,
                                relation.result_path(),
                            )?;
                            work.charge_traversal_work(prep_work, relation.result_path())?;
                        }
                        if prep_bytes > 0 {
                            iterator_bytes =
                                iterator_bytes.checked_add(prep_bytes).ok_or_else(|| {
                                    relational_read_denial(
                                        RelationalExecutionDenialCause::ChargedBytesOverflow,
                                        relation.result_path(),
                                    )
                                })?;
                            work.check_transient_window_bytes(
                                iterator_bytes,
                                relation.result_path(),
                            )?;
                        }
                    }
                    RelationalAdjacencyVisit::Acquired => {}
                    RelationalAdjacencyVisit::Relation(relation_id) => {
                        batch.push(relation_id);
                        if batch.len() >= ADJACENCY_BATCH_SIZE {
                            flush_adjacency_batch(
                                execution,
                                projection,
                                graph,
                                governance,
                                parameters,
                                relation,
                                relation_kind,
                                parent_id,
                                &mut batch,
                                &mut parent_targets,
                                &mut parent_predicate_count,
                                &mut predicate_sources,
                                work,
                                &mut cumulative_exec_work,
                            )?;
                        }
                    }
                }
                Ok(())
            })
            .map_err(|cause| relational_read_denial(cause, relation.result_path()))?;
        visit_res?;

        if !batch.is_empty() {
            flush_adjacency_batch(
                execution,
                projection,
                graph,
                governance,
                parameters,
                relation,
                relation_kind,
                parent_id,
                &mut batch,
                &mut parent_targets,
                &mut parent_predicate_count,
                &mut predicate_sources,
                work,
                &mut cumulative_exec_work,
            )?;
        }

        let sort_work = sort_work_units(parent_targets.len()).ok_or_else(|| {
            relational_read_denial(
                RelationalExecutionDenialCause::WorkCounterOverflow,
                relation.result_path(),
            )
        })?;
        if sort_work > 0 {
            debit_execution_work(
                execution,
                &mut cumulative_exec_work,
                sort_work,
                relation.result_path(),
            )?;
            work.charge_traversal_work(sort_work, relation.result_path())?;
        }
        parent_targets.sort_unstable_by_key(|target| {
            (
                target.relation.partition_id.0,
                target.relation.local_slot.0,
                target.relation.generation.0,
            )
        });
        validate_relation_cardinality(relation, parent_targets.len())?;
        counts.push(parent_targets.len());
        predicate_counts.push(parent_predicate_count);
        selected_targets.extend(parent_targets);
    }

    let child_ids: Vec<EntityId> = selected_targets.iter().map(|t| t.entity).collect();
    let mut children = project_nodes(
        execution,
        read,
        projection,
        graph,
        contract,
        governance,
        parameters,
        Some(relation.result_path_identity()),
        relation.result_path(),
        relation.child_entity(),
        &child_ids,
        work,
        collection_selection,
    )?;
    attach_occurrences(
        &mut children,
        &selected_targets,
        relation.result_path(),
        work,
    )?;
    distribute_relation_rows(
        parents,
        relation,
        counts,
        children,
        contract,
        governance,
        work,
        false,
        predicate_counts,
        predicate_sources,
    )
}

#[allow(clippy::too_many_arguments)]
fn flush_adjacency_batch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &VisibilityProjectionView,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    parameters: &WorthQueryAdmittedApplicationQueryParameters,
    relation: &WorthQueryInstalledGraphRelation,
    relation_kind: KindId,
    parent_id: EntityId,
    batch: &mut Vec<RelationId>,
    parent_targets: &mut Vec<RelationProjectionTarget>,
    parent_predicate_count: &mut usize,
    predicate_sources: &mut Vec<EntityId>,
    work: &mut ResultTreeWork,
    cumulative_exec_work: &mut u64,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let has_predicate = relation.predicate().is_some();
    let mut candidates = Vec::with_capacity(batch.len());

    for relation_id in batch.drain(..) {
        work.checkpoint(relation.result_path())?;
        debit_execution_work(execution, cumulative_exec_work, 1, relation.result_path())?;
        work.charge_adjacency(0, 1, 0, relation.result_path())?;
        let endpoints = projection
            .relation_record_with_projection_scope(
                execution,
                relation_id,
                ProjectionAspectScope::empty(),
                |record| {
                    (record.kind_id() == relation_kind
                        && record.lifecycle() == RecordLifecycleState::Live)
                        .then_some((record.source(), record.target()))
                },
            )
            .map_err(|cause| relational_read_denial(cause, relation.result_path()))?;
        let Some((source, target)) = endpoints else {
            continue;
        };
        let (matched_parent, child) = match relation.direction() {
            ApplicationQueryResultTraversalDirection::Forward => (source, target),
            ApplicationQueryResultTraversalDirection::Reverse => (target, source),
        };
        if matched_parent != parent_id {
            continue;
        }
        debit_execution_work(execution, cumulative_exec_work, 1, relation.result_path())?;
        work.charge_adjacency(0, 0, 1, relation.result_path())?;
        if has_predicate {
            work.charge_window_bytes(std::mem::size_of::<EntityId>(), relation.result_path())?;
            work.material_next(predicate_sources)?;
            predicate_sources.push(child);
            *parent_predicate_count = parent_predicate_count.saturating_add(1);
            candidates.push(RelationProjectionTarget {
                entity: child,
                relation: relation_id,
            });
        } else {
            work.charge_window_bytes(
                std::mem::size_of::<RelationProjectionTarget>(),
                relation.result_path(),
            )?;
            work.material_next(parent_targets)?;
            parent_targets.push(RelationProjectionTarget {
                entity: child,
                relation: relation_id,
            });
        }
    }

    if has_predicate && !candidates.is_empty() {
        retain_matching_targets(
            execution,
            projection,
            graph,
            governance,
            parameters,
            relation,
            &mut candidates,
            |target| target.entity,
            work,
        )?;
        for target in candidates {
            work.charge_window_bytes(
                std::mem::size_of::<RelationProjectionTarget>(),
                relation.result_path(),
            )?;
            work.material_next(parent_targets)?;
            parent_targets.push(target);
        }
    }
    Ok(())
}
