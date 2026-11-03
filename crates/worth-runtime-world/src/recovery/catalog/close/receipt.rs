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

    pub(super) fn copy_for_request(
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

    pub(super) fn matches_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        receipt: &RelationalCommitReceipt,
    ) -> Result<bool, Denial> {
        super::checkpoint(request, Self::heap_bytes(receipt)?)?;
        Ok(&self.receipt == receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_execution::{CancellationToken, SerialMemoryBudget, SerialRequest};

    #[test]
    fn recovered_receipt_storage_refuses_short_grants_and_stays_funded_until_drop() {
        // Descriptive data tests copying only; it issues no Native authority.
        let original = RelationalCommitReceipt {
            commit_id: CommitId(7),
            version_id: worth_relational::facade::identity::VersionId(8),
            // Keep receipt storage above the request's transient fixed scratch,
            // so the exact retained grant itself is the tested boundary.
            branch_id: BranchId("original-recovered-branch".repeat(4096)),
            parents: vec![CommitId(3), CommitId(5)],
        };
        let needed = original.branch_id.0.len() as u64 + 2 * std::mem::size_of::<CommitId>() as u64;
        for budget in [needed - 1, needed] {
            let memory = SerialMemoryBudget::new(Some(budget));
            let policy = SerialRequest::from_memory(memory.clone(), CancellationToken::new(), None);
            let request = ExecutionRequest::serial(&policy);
            let copied = RecoveredCloseReceipt::copy_for_request(&request, &original);
            if budget < needed {
                assert!(matches!(
                    copied,
                    Err(Denial::Execution(
                        worth_execution::WorkCeilingDenial::Admission(_)
                    ))
                ));
                assert_eq!(memory.charged_memory_bytes(), 0);
            } else {
                let copied = copied.unwrap();
                assert_eq!(copied.receipt(), &original);
                assert_eq!(memory.charged_memory_bytes(), needed);
                drop(copied);
                assert_eq!(memory.charged_memory_bytes(), 0);
            }
        }
    }

    #[test]
    fn recovered_receipt_copy_observes_the_original_work_stop_before_allocating() {
        let policy = worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(1 << 20),
                Some(0),
            ),
        );
        let policy = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
        let request = ExecutionRequest::serial(&policy);
        let original = RelationalCommitReceipt {
            commit_id: CommitId(7),
            version_id: worth_relational::facade::identity::VersionId(8),
            branch_id: BranchId("original-recovered-branch".into()),
            parents: vec![CommitId(3), CommitId(5)],
        };
        assert!(matches!(
            RecoveredCloseReceipt::copy_for_request(&request, &original),
            Err(Denial::Stopped(worth_execution::MapKernelStop::WorkCeiling))
        ));
        assert_eq!(policy.memory().unwrap().charged_memory_bytes(), 0);
    }
}
