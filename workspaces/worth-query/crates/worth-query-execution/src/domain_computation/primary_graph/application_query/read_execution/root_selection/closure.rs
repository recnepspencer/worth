//! One complete Native traversal, retaining real per-row discovery provenance.
use super::path_union::traversal_denial;
use super::*;
use crate::domain_computation::primary_graph::{
    schema_layout::WorthQueryPrimaryRelationLayout,
    structural_walk::{
        memory, StructuralWalkDependencies, StructuralWalkObservation, StructuralWalkQuery,
    },
};
use std::{cell::RefCell, collections::BTreeMap, sync::Arc};
use worth_query_installation::facade::WorthQueryInstalledRootPathStep;
use worth_relational::facade::{
    identity::EntityId,
    runtime::{PinnedStructuralWalkDenialKind, RelationalPinnedRead},
    transactions::RelationalExecutionDenialCause as Cause,
};

pub(super) fn advance(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPinnedRead,
    layout: &WorthQueryPrimaryRelationLayout,
    step: &WorthQueryInstalledRootPathStep,
    frontier: &BTreeMap<EntityId, RootPathSourceBuilder>,
    work: &mut RootSelectionWork<'_>,
    set_source: &mut Option<RootPathSourceBuilder>,
    path_index: usize,
    step_index: usize,
) -> Result<BTreeMap<EntityId, RootPathSourceBuilder>, WorthQueryApplicationReadExecutionDenial> {
    if layout.from != layout.to {
        return Err(traversal_denial(step.relation()));
    }
    let interruption = work.interruption;
    let original_stop = RefCell::new(None);
    let control = || match interruption.checkpoint(step.relation()) {
        Ok(()) => Ok(()),
        Err(stop) => {
            if original_stop.borrow().is_none() {
                *original_stop.borrow_mut() = Some(stop);
            }
            Err(Cause::NestedStopped)
        }
    };
    let result = execution.run_native_with_checkpoint(|context| {
        let bytes = frontier
            .len()
            .checked_mul(size_of::<EntityId>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| {
                super::super::denial::relational_read_denial(
                    Cause::ChargedBytesOverflow,
                    step.relation(),
                )
            })?;
        let _roots_memory =
            memory::reserve(execution, context.has_accounting(), bytes).map_err(|cause| {
                super::super::denial::relational_read_denial(cause, step.relation())
            })?;
        let mut roots = Vec::new();
        roots.try_reserve_exact(frontier.len()).map_err(|_| {
            super::super::denial::relational_read_denial(Cause::AllocationFailed, step.relation())
        })?;
        for root in frontier.keys() {
            work.checkpoint(step.relation())?;
            StructuralWalkObservation::checkpoint(context).map_err(|cause| {
                super::super::denial::relational_read_denial(cause, step.relation())
            })?;
            roots.push(*root);
        }
        let observed = StructuralWalkObservation::read(
            runtime,
            execution,
            StructuralWalkQuery {
                relation: step.relation(),
                roots: &roots,
                entity_kind: layout.from,
                relation_kind: layout.kind,
                direction: step.direction(),
                maximum_work: None,
                control: Some(&control),
            },
            context,
        );
        let (reads, units) = match &observed {
            Ok(body) => (body.native().reads(), body.native().work_units()),
            Err(stop) => (stop.reads, stop.work),
        };
        let charged = work.charge_walk(reads, units, step.relation());
        // Never erase acquired work, or replace the first caller's semantic stop
        // with the Native adapter's refusal carrier.
        if let Some(stop) = original_stop.borrow_mut().take() {
            return Err(stop);
        }
        charged?;
        let body = observed.map_err(|stop| match stop.kind {
            PinnedStructuralWalkDenialKind::Read(cause) => {
                super::super::denial::relational_read_denial(cause, step.relation())
            }
            PinnedStructuralWalkDenialKind::WorkLimitExceeded
            | PinnedStructuralWalkDenialKind::DistanceOverflow => read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                step.relation(),
            ),
            _ => traversal_denial(step.relation()),
        })?;
        work.checkpoint(step.relation())?;
        let mut step_reads = WorthQueryInvariantProjectionWork::default();
        step_reads.record_adjacencies(reads.adjacency_lists, reads.relation_records);
        step_reads.record_endpoints(reads.entity_records);
        work.record_step(
            execution,
            path_index,
            step_index,
            step.traversal(),
            step.direction(),
            step_reads,
            step.relation(),
        )?;
        let allocation = work.activate_source_allocation(execution)?;
        if let Some(source) = &mut *set_source {
            source.protect(execution, &allocation)?;
        }
        let prior = frontier
            .first_key_value()
            .and_then(|(_, source)| source.walks());
        let dependencies = StructuralWalkDependencies::append(prior, &body, execution, context)
            .map_err(|cause| {
                super::super::denial::relational_read_denial(cause, step.relation())
            })?;
        if let Some(source) = set_source {
            source
                .record_walk(&body, execution, context)
                .map_err(|cause| {
                    super::super::denial::relational_read_denial(cause, step.relation())
                })?;
            work.charge_source_copy(1, step.relation())?;
        }
        let mut next = BTreeMap::new();
        for visit in body.native().visits() {
            work.checkpoint(step.relation())?;
            StructuralWalkObservation::checkpoint(context).map_err(|cause| {
                super::super::denial::relational_read_denial(cause, step.relation())
            })?;
            let predecessor = frontier
                .get(&visit.root)
                .ok_or_else(|| traversal_denial(step.relation()))?;
            let same_prefix = match (prior, predecessor.walks()) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                _ => false,
            };
            if !same_prefix {
                return Err(traversal_denial(step.relation()));
            }
            work.charge_source_copy(predecessor.copy_cost() + 1, step.relation())?;
            work.admit_root_entry::<RootPathSourceBuilder>(execution, next.len())?;
            let mut source =
                predecessor.copy_for_step(execution, work.source_allocation.as_ref())?;
            source.observe_entity(execution, visit.entity.entity_id)?;
            source.set_walks(Arc::clone(&dependencies));
            next.insert(visit.entity.entity_id, source);
        }
        work.checkpoint(step.relation())?;
        Ok(next)
    });
    let result = match result {
        Ok((value, _)) => value,
        Err(stop) => Err(super::super::denial::relational_read_denial(
            stop.into(),
            step.relation(),
        )),
    };
    work.checkpoint(step.relation())?;
    result
}
