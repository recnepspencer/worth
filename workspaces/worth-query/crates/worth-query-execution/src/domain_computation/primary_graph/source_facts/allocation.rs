//! Original complete-walk backing for source metadata and its retained handoffs.
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
pub(in crate::domain_computation::primary_graph) struct SourceFactsAllocation {
    backing: Mutex<Backing>,
}

impl SourceFactsAllocation {
    pub(in crate::domain_computation::primary_graph) fn has_payment(&self) -> bool {
        self.backing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .ticket
            .is_some()
    }

    pub(in crate::domain_computation::primary_graph) fn open(
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Cause> {
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

    pub(in crate::domain_computation::primary_graph) fn claim(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        bytes: u64,
    ) -> Result<(), Cause> {
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

pub(super) fn tree_clone_bytes<T>(len: usize) -> Option<u64> {
    u64::try_from(len).ok()?.checked_mul(
        11_u64
            .checked_mul(size_of::<T>() as u64)?
            .checked_add(14 * size_of::<usize>() as u64)?,
    )
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
