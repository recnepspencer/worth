//! One execution reservation follows the page's newly owned payload until its last owner drops.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceCoordinates;
use std::alloc::Layout;
use worth_execution::{ExecutionAllocationDenial, ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::history::{BranchId, CommitId, RelationalCommitReceipt};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryCorrectionHistoryPageAllocation {
    memory: Option<ExecutionMemoryReservation>,
}
type Denial = WorthQueryAftermathCausalityReadDenial;
impl WorthQueryCorrectionHistoryPageAllocation {
    pub(super) fn new(execution: &ExecutionRequest<'_, '_>) -> Result<Self, Denial> {
        Ok(Self {
            memory: execution
                .allocation_policy()
                .reserve_memory(0)
                .map_err(allocation_denial)?,
        })
    }

    /// Reserves the concrete result capacity before allocating it, without a separate quota.
    pub(in crate::domain_computation::primary_graph) fn reserve_projection_slots<T>(
        &mut self,
        count: usize,
    ) -> Result<Vec<T>, Denial> {
        let requested = slots::<T>(count)?;
        self.grow(requested)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| allocation_failed())?;
        let actual = slots::<T>(values.capacity())?;
        self.grow(actual.checked_sub(requested).ok_or_else(overflow)?)?;
        Ok(values)
    }

    /// Fact payloads moved into projected references remain funded; only the old slots go away.
    pub(in crate::domain_computation::primary_graph) fn release_fact_slots(
        &mut self,
        capacity: usize,
    ) -> Result<(), Denial> {
        let released = u64::try_from(slots::<WorthQueryCanonicalCorrectionFact>(capacity)?)
            .map_err(|_| overflow())?;
        if let Some(memory) = &mut self.memory {
            let remaining = memory.bytes().checked_sub(released).ok_or_else(overflow)?;
            memory
                .resize(remaining)
                .map_err(|cause| allocation_denial(cause.into()))?;
        }
        Ok(())
    }

    /// The root receipt is newly cloned. Target names and native bases share existing backing.
    pub(in crate::domain_computation::primary_graph) fn clone_root(
        &mut self,
        root: &WorthQueryRecordedCorrectionSource,
    ) -> Result<WorthQueryRecordedCorrectionSource, Denial> {
        Ok(WorthQueryRecordedCorrectionSource::from_untrusted_parts(
            root.world().clone(),
            self.clone_receipt(root.relational())?,
        ))
    }

    /// Copies decoded transient metadata into funded page custody, including every owned receipt.
    pub(super) fn clone_fact(
        &mut self,
        fact: &WorthQueryCanonicalCorrectionFact,
    ) -> Result<WorthQueryCanonicalCorrectionFact, Denial> {
        Ok(WorthQueryCanonicalCorrectionFact {
            role: fact.role,
            parent: self.clone_receipt(&fact.parent)?,
            child: self.clone_receipt(&fact.child)?,
            record: fact.record.clone(),
            source: self.clone_coordinates(&fact.source)?,
            root: self.clone_coordinates(&fact.root)?,
            selected: fact
                .selected
                .as_ref()
                .map(|value| self.clone_coordinates(value))
                .transpose()?,
            active: fact
                .active
                .as_ref()
                .map(|value| self.clone_coordinates(value))
                .transpose()?,
            prior: fact
                .prior
                .as_ref()
                .map(|value| self.clone_coordinates(value))
                .transpose()?,
        })
    }

    fn clone_coordinates(
        &mut self,
        source: &WorthQueryCorrectionSourceCoordinates,
    ) -> Result<WorthQueryCorrectionSourceCoordinates, Denial> {
        Ok(WorthQueryCorrectionSourceCoordinates::from_untrusted_parts(
            source.world_owner(),
            source.world_commit(),
            self.clone_receipt(source.relational())?,
        ))
    }
    fn clone_receipt(
        &mut self,
        receipt: &RelationalCommitReceipt,
    ) -> Result<RelationalCommitReceipt, Denial> {
        let mut branch = self.reserve_projection_slots::<u8>(receipt.branch_id.0.len())?;
        branch.extend_from_slice(receipt.branch_id.0.as_bytes());
        let mut parents = self.reserve_projection_slots::<CommitId>(receipt.parents.len())?;
        parents.extend_from_slice(&receipt.parents);
        Ok(RelationalCommitReceipt {
            commit_id: receipt.commit_id,
            version_id: receipt.version_id,
            branch_id: BranchId(
                String::from_utf8(branch).expect("a copied branch name remains UTF-8"),
            ),
            parents,
        })
    }
    fn grow(&mut self, bytes: usize) -> Result<(), Denial> {
        let bytes = u64::try_from(bytes).map_err(|_| overflow())?;
        if let Some(memory) = &mut self.memory {
            let total = memory.bytes().checked_add(bytes).ok_or_else(overflow)?;
            memory
                .grow_for_allocation(total)
                .map_err(allocation_denial)?;
        }
        Ok(())
    }
}
fn slots<T>(count: usize) -> Result<usize, Denial> {
    Layout::array::<T>(count)
        .map(|layout| layout.size())
        .map_err(|_| overflow())
}
fn allocation_denial(cause: ExecutionAllocationDenial) -> Denial {
    match cause.classify() {
        worth_execution::ExecutionAllocationDenialClass::Request(cause) => {
            aftermath_read_stop(cause.into())
        }
        worth_execution::ExecutionAllocationDenialClass::Physical(_) => allocation_failed(),
    }
}
fn overflow() -> Denial {
    aftermath_read_stop(worth_relational::facade::transactions::RelationalExecutionDenialCause::ChargedBytesOverflow)
}
fn allocation_failed() -> Denial {
    aftermath_read_stop(
        worth_relational::facade::transactions::RelationalExecutionDenialCause::AllocationFailed,
    )
}

#[cfg(test)]
mod tests;
