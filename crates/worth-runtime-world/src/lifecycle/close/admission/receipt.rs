//! A descriptive recovered receipt owns exactly its original-request funding.
use crate::lifecycle::RuntimeWorldCloseDenial as Denial;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::history::{BranchId, CommitId, RelationalCommitReceipt};

#[derive(Debug)]
pub(crate) struct RecoveredCloseReceipt {
    receipt: RelationalCommitReceipt,
    _funding: Option<ExecutionMemoryReservation>,
}

impl RecoveredCloseReceipt {
    fn heap_bytes(receipt: &RelationalCommitReceipt) -> Result<u64, Denial> {
        receipt
            .parents
            .len()
            .checked_mul(std::mem::size_of::<CommitId>())
            .and_then(|bytes| bytes.checked_add(receipt.branch_id.0.len()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::ReportAllocationFailed)
    }

    pub(crate) fn copy_for_request(
        request: &ExecutionRequest<'_, '_>,
        receipt: &RelationalCommitReceipt,
    ) -> Result<Self, Denial> {
        let heap = Self::heap_bytes(receipt)?;
        super::checkpoint(request, heap)?;
        let funding = request
            .reserve_retained_memory(heap)
            .map_err(Denial::Execution)?;
        let mut branch = String::new();
        let mut parents = Vec::new();
        branch
            .try_reserve_exact(receipt.branch_id.0.len())
            .map_err(|_| Denial::ReportAllocationFailed)?;
        parents
            .try_reserve_exact(receipt.parents.len())
            .map_err(|_| Denial::ReportAllocationFailed)?;
        if branch.capacity() != receipt.branch_id.0.len()
            || parents.capacity() != receipt.parents.len()
        {
            return Err(Denial::ReportAllocationFailed);
        }
        branch.push_str(&receipt.branch_id.0);
        parents.extend_from_slice(&receipt.parents);
        Ok(Self {
            receipt: RelationalCommitReceipt {
                commit_id: receipt.commit_id,
                version_id: receipt.version_id,
                branch_id: BranchId(branch),
                parents,
            },
            _funding: funding,
        })
    }

    pub(crate) fn receipt(&self) -> &RelationalCommitReceipt {
        &self.receipt
    }

    pub(crate) fn matches_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        receipt: &RelationalCommitReceipt,
    ) -> Result<bool, Denial> {
        super::checkpoint(request, Self::heap_bytes(receipt)?)?;
        Ok(&self.receipt == receipt)
    }
}
