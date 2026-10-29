//! Original managed-source custody before framework tree and attachment growth.
use super::WorthQueryApplicationReadExecutionDenial as Denial;
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(super) struct MaterialTreeStorage<'request, 'authority> {
    execution: ExecutionRequest<'request, 'authority>,
    allocation: Arc<SourceFactsAllocation>,
}
impl<'request, 'authority> MaterialTreeStorage<'request, 'authority> {
    pub(super) fn new(
        execution: ExecutionRequest<'request, 'authority>,
        allocation: Arc<SourceFactsAllocation>,
    ) -> Self {
        Self {
            execution,
            allocation,
        }
    }
    fn denied(cause: Cause) -> Denial {
        super::super::denial::relational_read_denial(cause, "managed source tree")
    }
    pub(super) fn bytes(&self, bytes: usize) -> Result<(), Denial> {
        self.allocation
            .claim(
                &self.execution,
                u64::try_from(bytes).map_err(|_| Self::denied(Cause::ChargedBytesOverflow))?,
            )
            .map_err(Self::denied)
    }
    fn work(&self, units: usize) -> Result<(), Denial> {
        let units = u64::try_from(units).map_err(|_| Self::denied(Cause::WorkCounterOverflow))?;
        let mut returned = None;
        let outcome = self.execution.run_native_with_checkpoint(|context| {
            returned = Some(if context.has_accounting() {
                context.checkpoint(units)
            } else {
                context.poll()
            });
        });
        match (outcome, returned) {
            (Ok(_), Some(result)) => result.map_err(|cause| Self::denied(cause.into())),
            (
                Err(worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
                Some(Err(cause)),
            ) => Err(Self::denied(cause.into())),
            (Err(cause), _) => Err(Self::denied(cause.into())),
            (Ok(_), None) => unreachable!("completed managed tree work"),
        }
    }
    pub(super) fn slots<T>(&self, count: usize) -> Result<(), Denial> {
        self.allocation
            .claim_slice::<T>(&self.execution, count)
            .map_err(Self::denied)
    }
    pub(super) fn tree<T>(&self, len: usize) -> Result<(), Denial> {
        self.allocation
            .claim_tree::<T>(&self.execution, len)
            .map_err(Self::denied)
    }
    pub(super) fn field(&self, slot: &str, value: &AspectValue) -> Result<(), Denial> {
        self.text(slot)?;
        self.bytes(value.owned_allocation_capacity_bytes())?;
        self.work(value.owned_allocation_capacity_bytes())
    }
    pub(super) fn text(&self, text: &str) -> Result<(), Denial> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(
                std::alloc::Layout::array::<u8>(text.len())
                    .map_err(|_| Self::denied(Cause::ChargedBytesOverflow))?,
            )
            .map_err(|_| Self::denied(Cause::ChargedBytesOverflow))?
            .0
            .pad_to_align();
        self.bytes(layout.size())?;
        self.work(text.len())
    }
}
