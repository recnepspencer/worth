//! Fund executed-step descriptors once, independently of terminal row count.
use super::*;
use crate::domain_computation::primary_graph::application_query::access_receipt::{
    RootPathReadEvidence, WorthQueryApplicationQueryRootPathStepRead as Step,
};
use crate::domain_computation::primary_graph::structural_walk::memory;
use std::sync::Arc;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryRootPathTraversal, RelationDirection,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

impl RootSelectionWork<'_> {
    pub(super) fn reserve_steps(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        count: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.checkpoint(subject)?;
        let result = execution.run_native_with_checkpoint(|context| {
            let bytes = count
                .checked_mul(size_of::<Step>())
                .and_then(|n| {
                    n.checked_add(size_of::<RootPathReadEvidence>() + 2 * size_of::<usize>())
                })
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(Cause::ChargedBytesOverflow)?;
            self.step_memory = memory::reserve(execution, context.has_accounting(), bytes)?;
            self.steps
                .try_reserve_exact(count)
                .map_err(|_| Cause::AllocationFailed)?;
            self.step_limit = count;
            Ok::<(), Cause>(())
        });
        let result = match result {
            Ok((value, _)) => value,
            Err(stop) => Err(stop.into()),
        };
        self.checkpoint(subject)?;
        result.map_err(|cause| super::super::denial::relational_read_denial(cause, subject))
    }

    pub(super) fn record_step(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        path_index: usize,
        step_index: usize,
        traversal: ApplicationQueryRootPathTraversal,
        direction: RelationDirection,
        reads: WorthQueryInvariantProjectionWork,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.checkpoint(subject)?;
        let result = execution.run_native_with_checkpoint(|_context| {
            if self.steps.len() >= self.step_limit {
                return Err(Cause::ExpectedIdentitiesNotCanonical);
            }
            self.steps.push(Step::new(
                path_index, step_index, traversal, direction, reads,
            ));
            Ok::<(), Cause>(())
        });
        let result = match result {
            Ok((value, _)) => value,
            Err(stop) => Err(stop.into()),
        };
        self.checkpoint(subject)?;
        result.map_err(|cause| super::super::denial::relational_read_denial(cause, subject))
    }

    pub(super) fn take_steps(&mut self) -> Option<Arc<RootPathReadEvidence>> {
        (!self.steps.is_empty()).then(|| {
            Arc::new(RootPathReadEvidence::new(
                std::mem::take(&mut self.steps),
                self.step_memory.take(),
            ))
        })
    }

    pub(super) fn charge_walk(
        &mut self,
        reads: worth_relational::facade::runtime::PinnedStructuralWalkReads,
        work: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        // Acquired Native work survives a later caller stop or charge refusal.
        self.reads
            .record_adjacencies(reads.adjacency_lists, reads.relation_records);
        self.reads.record_endpoints(reads.entity_records);
        self.work_units = self.work_units.checked_add(work).ok_or_else(|| {
            read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                subject,
            )
        })?;
        if let Some(spent) = self.spent {
            spent.charge(work, subject)?;
        }
        self.checkpoint(subject)
    }
}
