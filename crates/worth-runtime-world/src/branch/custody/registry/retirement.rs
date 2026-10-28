//! Funded drain storage, with the actual custody roster guarded through effect.
use super::*;
use crate::branch::{FundedOwnerRetirementWork, RuntimeWorldBranchReplacementDenial as Denial};
use std::sync::{MutexGuard, TryLockError};
use worth_execution::{ExecutionRequest, MapKernelContext, WorkCeilingDenial};

impl OwnerCreatedComponentCustodyRegistry {
    pub(crate) fn with_retirement_work<R>(
        &self,
        branch: &ProductBranchIdentity,
        incarnation: ProductBranchIncarnation,
        request: ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        retire: impl FnOnce(&mut MapKernelContext<'_, '_>) -> Result<R, Denial>,
    ) -> Result<(R, FundedOwnerRetirementWork), Denial> {
        let count = {
            let state = self.try_state()?;
            roster_work(&state, branch, 1, context)?;
            matching(&state, branch, incarnation)
        };
        let mut output = reserve_work(count, request)?;
        let mut state = self.try_state()?;
        // Both the count and drain compare real variable branch keys. Nothing
        // can add a custody record while this guard protects the transition.
        roster_work(&state, branch, 2, context)?;
        if matching(&state, branch, incarnation) > output.work.capacity() {
            return Err(Denial::CapacityExhausted);
        }
        let retired = retire(context)?;
        let mut index = 0;
        while index < state.installed.len() {
            let record = &state.installed[index];
            if record.product_branch() == branch && record.incarnation() == incarnation {
                output
                    .work
                    .push(state.installed.swap_remove(index).into_retirement_work());
            } else {
                index += 1;
            }
        }
        drop(state);
        Ok((retired, output))
    }

    fn try_state(&self) -> Result<MutexGuard<'_, CustodyRegistryState>, Denial> {
        match self.state.try_lock() {
            Ok(guard) => Ok(guard),
            Err(TryLockError::Poisoned(error)) => Ok(error.into_inner()),
            Err(TryLockError::WouldBlock) => Err(Denial::CustodyBusy),
        }
    }
}

fn matching(
    state: &CustodyRegistryState,
    branch: &ProductBranchIdentity,
    incarnation: ProductBranchIncarnation,
) -> usize {
    state
        .installed
        .iter()
        .filter(|record| record.product_branch() == branch && record.incarnation() == incarnation)
        .count()
}

fn roster_work(
    state: &CustodyRegistryState,
    branch: &ProductBranchIdentity,
    scans: u64,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Denial> {
    let width = u64::try_from(branch.name().as_str().len())
        .ok()
        .and_then(|n| n.checked_add(std::mem::size_of::<ProductBranchIdentity>() as u64))
        .and_then(|n| n.checked_add(std::mem::size_of::<ProductBranchIncarnation>() as u64))
        .ok_or(Denial::WorkCounterOverflow)?;
    let work = u64::try_from(state.installed.len())
        .ok()
        .and_then(|n| n.checked_mul(width))
        .and_then(|n| n.checked_mul(scans))
        .ok_or(Denial::WorkCounterOverflow)?;
    if context.has_accounting() {
        context.checkpoint(work)
    } else {
        context.poll()
    }
    .map_err(|cause| Denial::Request(WorkCeilingDenial::Stopped(cause)))
}

fn reserve_work(
    count: usize,
    request: ExecutionRequest<'_, '_>,
) -> Result<FundedOwnerRetirementWork, Denial> {
    let quote = bytes(count)?;
    let memory = request
        .reserve_retained_memory(quote)
        .map_err(Denial::Request)?;
    let mut output = FundedOwnerRetirementWork {
        work: Vec::new(),
        _memory: memory,
    };
    output
        .work
        .try_reserve_exact(count)
        .map_err(Denial::BackingAllocation)?;
    if let Some(memory) = output._memory.as_mut() {
        memory
            .resize(bytes(output.work.capacity())?)
            .map_err(|cause| {
                Denial::Request(WorkCeilingDenial::Admission(
                    worth_execution::LeaseDenial::MemoryExhausted(cause),
                ))
            })?;
    }
    Ok(output)
}

fn bytes(count: usize) -> Result<u64, Denial> {
    let layout = std::alloc::Layout::array::<OwnerRetirementWork>(count)
        .map_err(|_| Denial::SizeOverflow)?;
    u64::try_from(layout.size()).map_err(|_| Denial::SizeOverflow)
}
