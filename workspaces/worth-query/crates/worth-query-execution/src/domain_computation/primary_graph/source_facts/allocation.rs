//! Original complete-source backing for walk or material-selection custody.
use crate::domain_computation::vocabulary::primary_graph::storage::tree_clone_bytes;
use super::super::structural_walk::{memory, StructuralWalkObservation};
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

#[derive(Debug)]
struct Backing {
    bytes: u64,
    ticket: Option<ExecutionMemoryReservation>,
}

/// One conservative read-batch grant, retained by its actual source witnesses.
#[derive(Debug)]
pub struct SourceFactsAllocation {
    backing: Mutex<Backing>,
}

impl SourceFactsAllocation {
    pub(in crate::domain_computation) fn work(
        execution: &ExecutionRequest<'_, '_>,
        units: u64,
    ) -> Result<(), Cause> {
        Self::request_work(execution, units).map_err(Into::into)
    }
    pub(in crate::domain_computation) fn request_work(
        execution: &ExecutionRequest<'_, '_>,
        units: u64,
    ) -> Result<(), worth_execution::WorkCeilingDenial> {
        let mut result = None;
        let outcome = execution.run_native_with_checkpoint(|context| {
            result = Some(if context.has_accounting() {
                context.checkpoint(units)
            } else {
                context.poll()
            });
        });
        match (outcome, result) {
            (Ok(_), Some(result)) => result.map_err(worth_execution::WorkCeilingDenial::Stopped),
            (
                Err(worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
                Some(Err(cause)),
            ) => Err(worth_execution::WorkCeilingDenial::Stopped(cause)),
            (Err(cause), _) => Err(cause),
            (Ok(_), None) => unreachable!("completed source metadata checkpoint"),
        }
    }
    pub fn claim_arc<T>(&self, execution: &ExecutionRequest<'_, '_>) -> Result<(), Cause> {
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<T>())
            .ok()
            .map(|(layout, _)| layout.pad_to_align().size())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        self.claim(execution, bytes)
    }
    pub(in crate::domain_computation::primary_graph) fn has_payment(&self) -> bool {
        self.backing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .ticket
            .is_some()
    }

    pub fn open(execution: &ExecutionRequest<'_, '_>) -> Result<Arc<Self>, Cause> {
        let bytes = (size_of::<Self>() + 2 * size_of::<usize>()) as u64;
        let result = execution.run_native_with_checkpoint(|context| {
            memory::reserve(execution, context.has_accounting(), bytes)
        });
        let ticket = match result {
            Ok((value, _)) => value,
            Err(stop) => Err(stop.into()),
        }?;
        Ok(Arc::new(Self {
            backing: Mutex::new(Backing { bytes, ticket }),
        }))
    }

    pub fn claim(&self, execution: &ExecutionRequest<'_, '_>, bytes: u64) -> Result<(), Cause> {
        let result = execution.run_native_with_checkpoint(|context| {
            StructuralWalkObservation::checkpoint(context)?;
            let mut backing = self
                .backing
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let next = backing
                .bytes
                .checked_add(bytes)
                .ok_or(Cause::ChargedBytesOverflow)?;
            if backing.ticket.is_some() {
                memory::resize(&mut backing.ticket, next)?;
            } else {
                backing.ticket = memory::reserve(execution, context.has_accounting(), next)?;
            }
            backing.bytes = next;
            Ok::<(), Cause>(())
        });
        match result {
            Ok((value, _)) => value,
            Err(stop) => Err(stop.into()),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn claim_tree<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        len: usize,
    ) -> Result<(), Cause> {
        let bytes = tree_insertion_bytes::<T>(len).ok_or(Cause::ChargedBytesOverflow)?;
        self.claim(execution, bytes)
    }

    pub(in crate::domain_computation::primary_graph) fn claim_slice<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        len: usize,
    ) -> Result<(), Cause> {
        let bytes = len
            .checked_mul(size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        self.claim(execution, bytes)
    }

    pub(in crate::domain_computation::primary_graph) fn claim_tree_copy<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        len: usize,
    ) -> Result<(), Cause> {
        self.claim(
            execution,
            tree_clone_bytes::<T>(len).ok_or(Cause::ChargedBytesOverflow)?,
        )
    }
}

fn tree_insertion_bytes<T>(len: usize) -> Option<u64> {
    let after = len.checked_add(1)?;
    let levels = u64::from(usize::BITS - after.leading_zeros()) + 1;
    levels.checked_mul(
        11_u64
            .checked_mul(size_of::<T>() as u64)?
            .checked_add(14 * size_of::<usize>() as u64)?,
    )
}
