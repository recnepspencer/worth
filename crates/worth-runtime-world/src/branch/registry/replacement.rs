//! One registry transition guarded by both actual reference images.
use super::*;
use crate::branch::RuntimeWorldBranchReplacementDenial as Denial;
use std::sync::TryLockError;

impl ProductBranchRegistry {
    pub(crate) fn retire_replaced_head(
        &self,
        old: &ProductBranchObservation,
        candidate: &ProductBranchObservation,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<
        (
            ProductBranchReferenceCell,
            ProductBranchReferenceRetirement,
            Option<crate::identity::CompositeCommitIdentity>,
        ),
        Denial,
    > {
        let mut state = match self.state.try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
            Err(TryLockError::WouldBlock) => return Err(Denial::RegistryBusy),
        };
        if old.owner_identity() != state.owner || candidate.owner_identity() != state.owner {
            return Err(Denial::ForeignOwner);
        }
        // Hash and ordered-index lookup/removal are bounded by the actual
        // occupied roster. All variable comparison work precedes the probes.
        let width = old
            .branch_identity()
            .name()
            .as_str()
            .len()
            .checked_add(candidate.branch_identity().name().as_str().len())
            .and_then(|n| {
                n.checked_add(std::mem::size_of::<crate::identity::CompositeBasisKey>() * 2)
            })
            .and_then(|n| n.checked_add(std::mem::size_of::<ProductBranchIdentity>() * 2))
            .ok_or(Denial::WorkCounterOverflow)?;
        let work = (state.entries.len() as u64)
            .checked_add(1)
            .and_then(|n| n.checked_mul(width as u64))
            // Two exact probes, exact+ordered removal, occurrence removal,
            // and the optional root comparison.
            .and_then(|n| n.checked_mul(6))
            .ok_or(Denial::WorkCounterOverflow)?;
        if context.has_accounting() {
            context.checkpoint(work)
        } else {
            context.poll()
        }
        .map_err(|cause| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
        let old_entry = state
            .entries
            .get(old.branch_identity())
            .filter(|entry| entry.lifecycle == old.lifecycle_incarnation())
            .ok_or(Denial::OldRetired)?;
        let old_cell = old_entry.cell.clone();
        let boundary = old_entry.retirement_boundary.clone();
        let candidate_cell = state
            .entries
            .get(candidate.branch_identity())
            .filter(|entry| entry.lifecycle == candidate.lifecycle_incarnation())
            .ok_or(Denial::CandidateRetired)?
            .cell
            .clone();
        if old.branch_identity() == candidate.branch_identity() {
            return Err(Denial::SameOccurrence);
        }
        let retirement = old_cell.try_retire_pair(old, &candidate_cell, candidate, execution)?;
        // Pair guards have dropped. Reacquiring this now-tombstoned old cell
        // cannot retire a later head. Registry removal allocates nothing.
        let entry = release_installed_entry(&mut state, old.branch_identity())
            .expect("the guarded occurrence is still installed under this mutex");
        drop(state);
        drop(candidate_cell);
        drop(entry);
        Ok((old_cell, retirement, boundary))
    }
}
