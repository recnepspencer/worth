//! Original custody only for footprint backing retained by a managed source image.
use super::*;
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(super) struct MaterialFootprintStorage<'a, 'request, 'authority> {
    execution: &'a ExecutionRequest<'request, 'authority>,
    allocation: Option<&'a Arc<SourceFactsAllocation>>,
}
impl<'a, 'request, 'authority> MaterialFootprintStorage<'a, 'request, 'authority> {
    pub(super) fn new(
        execution: &'a ExecutionRequest<'request, 'authority>,
        allocation: Option<&'a Arc<SourceFactsAllocation>>,
    ) -> Self {
        Self {
            execution,
            allocation,
        }
    }
    fn denied(cause: Cause) -> WorthQueryApplicationReadExecutionDenial {
        super::super::super::denial::relational_read_denial(cause, "managed source footprint")
    }
    fn claim(&self, bytes: usize) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if let Some(allocation) = self.allocation {
            let bytes =
                u64::try_from(bytes).map_err(|_| Self::denied(Cause::ChargedBytesOverflow))?;
            allocation
                .claim(self.execution, bytes)
                .map_err(Self::denied)?;
        }
        Ok(())
    }
    pub(super) fn slots<T>(
        &self,
        count: usize,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if self.allocation.is_some() {
            self.claim(
                count
                    .checked_mul(size_of::<T>())
                    .ok_or_else(|| Self::denied(Cause::ChargedBytesOverflow))?,
            )?;
        }
        Ok(())
    }
    pub(super) fn next<T>(
        &self,
        values: &mut Vec<T>,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if self.allocation.is_some() && values.len() == values.capacity() {
            self.slots::<T>(
                values
                    .len()
                    .checked_add(1)
                    .ok_or_else(|| Self::denied(Cause::ChargedBytesOverflow))?,
            )?;
            values
                .try_reserve_exact(1)
                .map_err(|_| Self::denied(Cause::AllocationFailed))?;
        }
        Ok(())
    }
    pub(super) fn field(
        &self,
        values: &mut Vec<WorthQueryObservedFieldRevision>,
        entity: &str,
        aspect: &worth_foundational::facade::AspectKey,
        field: &worth_foundational::facade::FieldKey,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if self.allocation.is_some() {
            let bytes = entity
                .len()
                .checked_add(aspect.owned_allocation_capacity_bytes())
                .and_then(|bytes| bytes.checked_add(field.owned_allocation_capacity_bytes()))
                .ok_or_else(|| Self::denied(Cause::ChargedBytesOverflow))?;
            self.claim(bytes)?;
            self.work(bytes)?;
        }
        self.next(values)
    }
    pub(super) fn field_copy(
        &self,
        values: &mut Vec<WorthQueryObservedFieldRevision>,
        field: &WorthQueryObservedFieldRevision,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if self.allocation.is_some() {
            self.claim(field.owned_capacity_bytes())?;
            self.work(field.owned_capacity_bytes())?;
        }
        self.next(values)
    }
    pub(super) fn work(
        &self,
        units: usize,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if self.allocation.is_none() {
            return Ok(());
        }
        let units = u64::try_from(units).map_err(|_| Self::denied(Cause::WorkCounterOverflow))?;
        let mut result = None;
        let outcome = self.execution.run_native_with_checkpoint(|context| {
            result = Some(if context.has_accounting() {
                context.checkpoint(units)
            } else {
                context.poll()
            });
        });
        match (outcome, result) {
            (Ok(_), Some(result)) => result.map_err(|cause| Self::denied(cause.into())),
            (
                Err(worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
                Some(Err(cause)),
            ) => Err(Self::denied(cause.into())),
            (Err(cause), _) => Err(Self::denied(cause.into())),
            (Ok(_), None) => unreachable!("completed source footprint checkpoint"),
        }
    }
    pub(super) fn normalize(
        &self,
        footprint: &mut WorthQueryObservedSourceFootprint,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if self.allocation.is_none() {
            super::normalize_source_footprint(footprint);
            return Ok(());
        }
        super::material_ordering::normalize(self, footprint)
    }
    pub(super) fn execution(&self) -> &ExecutionRequest<'request, 'authority> {
        self.execution
    }
}
