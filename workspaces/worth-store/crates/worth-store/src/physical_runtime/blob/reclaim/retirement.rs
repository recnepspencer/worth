use std::time::{Duration, Instant};

use crate::physical_runtime::{
    durability::{DisplacedArtifact, RetiredArtifact},
    PhysicalRetirementDenial, ServingPhysicalRuntime,
};

use super::{
    BlobReclaimContinuationFailure, BlobReclaimDisplacedExtent, BlobReclaimReceipt,
    BlobReclaimRetirement, BlobReclaimRetirementBudget,
};

impl ServingPhysicalRuntime {
    /// Continue only this Store's already-published reclaim. This performs no
    /// selected scan and cannot republish a manifest or descriptor.
    pub(in crate::physical_runtime) fn continue_blob_reclaim_retirement(
        &self,
        receipt: &mut BlobReclaimReceipt,
        budget: BlobReclaimRetirementBudget,
    ) -> Result<BlobReclaimRetirement, BlobReclaimContinuationFailure> {
        if receipt.store != self.store_identity() || receipt.runtime != self.runtime_identity() {
            return Err(BlobReclaimContinuationFailure::ForeignStoreOrRuntime);
        }
        self.blobs()
            .map_err(|_| BlobReclaimContinuationFailure::ServingRequiresInspection)?;
        retire_batch(self, receipt, budget);
        Ok(receipt.retirement)
    }
}

/// Every native retirement attempt consumes work, even when it services an
/// unrelated FIFO artifact. Only exact completed extents can change credit.
pub(super) fn retire_batch(
    runtime: &ServingPhysicalRuntime,
    receipt: &mut BlobReclaimReceipt,
    budget: BlobReclaimRetirementBudget,
) {
    if !matches!(
        receipt.retirement,
        BlobReclaimRetirement::AwaitingRetirement | BlobReclaimRetirement::Pending(_)
    ) {
        return;
    }
    reconcile_completed_extents(runtime, receipt);
    if receipt.completed.iter().all(|completed| *completed) {
        receipt.retirement = BlobReclaimRetirement::Completed;
        return;
    }
    let started = Instant::now();
    let duration = Duration::from_millis(budget.deadline().signal_deadline().get());
    for _ in 0..budget.maximum_work() {
        if started.elapsed() >= duration {
            break;
        }
        let retired = match runtime.retire_displaced_artifact() {
            Ok(Some(value)) => value,
            Ok(None) => {
                reconcile_completed_extents(runtime, receipt);
                receipt.retirement = if receipt.completed.iter().all(|completed| *completed) {
                    BlobReclaimRetirement::Completed
                } else {
                    BlobReclaimRetirement::Pending(PhysicalRetirementDenial::Absent)
                };
                return;
            }
            Err(cause) => {
                reconcile_completed_extents(runtime, receipt);
                receipt.retirement = if receipt.completed.iter().all(|completed| *completed) {
                    BlobReclaimRetirement::Completed
                } else {
                    BlobReclaimRetirement::Pending(cause)
                };
                return;
            }
        };
        credit_exact_retirement(
            &receipt.displaced,
            &mut receipt.completed,
            &mut receipt.bytes_released,
            retired,
        );
        if receipt.completed.iter().all(|completed| *completed) {
            receipt.retirement = BlobReclaimRetirement::Completed;
            return;
        }
    }
    reconcile_completed_extents(runtime, receipt);
    if receipt.completed.iter().all(|completed| *completed) {
        receipt.retirement = BlobReclaimRetirement::Completed;
    }
}

/// The native retention owner records completion only after its durable
/// retirement protocol settles. Another caller may have finished a member of
/// this receipt while this caller was pending or servicing the shared FIFO.
fn reconcile_completed_extents(runtime: &ServingPhysicalRuntime, receipt: &mut BlobReclaimReceipt) {
    for (index, extent) in receipt.displaced.iter().enumerate() {
        if receipt.completed[index] {
            continue;
        }
        let expected = DisplacedArtifact {
            source_root: extent.source_root,
            artifact: RetiredArtifact::Extent {
                extent: extent.extent,
                generation: extent.generation,
                range: extent.range,
            },
            bytes: extent.bytes,
        };
        if let Some(completed) = runtime.completed_displaced_exact(expected) {
            credit_exact_retirement(
                &receipt.displaced,
                &mut receipt.completed,
                &mut receipt.bytes_released,
                completed,
            );
        }
    }
}

/// Credits the native completed fact once; a matching extent tuple alone is
/// not a substitute for that owner's durable retirement completion.
fn credit_exact_retirement(
    displaced: &[BlobReclaimDisplacedExtent],
    completed: &mut [bool],
    bytes_released: &mut u64,
    retired: DisplacedArtifact,
) -> bool {
    let RetiredArtifact::Extent {
        extent,
        generation,
        range,
    } = retired.artifact
    else {
        return false;
    };
    let Ok(index) = displaced.binary_search_by_key(&(extent, generation, range), extent_key) else {
        return false;
    };
    if completed[index]
        || displaced[index].source_root != retired.source_root
        || displaced[index].bytes != retired.bytes
    {
        return false;
    }
    *bytes_released = bytes_released
        .checked_add(retired.bytes)
        .expect("pre-admitted reclaim displacement bytes fit in u64");
    completed[index] = true;
    true
}

pub(super) fn extent_key(
    value: &BlobReclaimDisplacedExtent,
) -> (u64, u64, worth_store_physical_format::ExtentArenaRange) {
    (value.extent, value.generation, value.range)
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::{ExtentArenaId, ExtentArenaRange};

    fn extent(id: u64) -> (BlobReclaimDisplacedExtent, DisplacedArtifact) {
        let range = ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), id * 64, 64).unwrap();
        (
            BlobReclaimDisplacedExtent {
                extent: id,
                generation: 1,
                range,
                source_root: 1,
                bytes: 64,
            },
            DisplacedArtifact {
                source_root: 1,
                artifact: RetiredArtifact::Extent {
                    extent: id,
                    generation: 1,
                    range,
                },
                bytes: 64,
            },
        )
    }

    #[test]
    fn exact_native_retirements_credit_once_and_only_finish_with_all_bits() {
        let (first, native_first) = extent(1);
        let (second, native_second) = extent(2);
        let (_, unrelated) = extent(3);
        let displaced = [first, second];
        let mut completed = [false, false];
        let mut bytes = 0;
        assert!(!credit_exact_retirement(
            &displaced,
            &mut completed,
            &mut bytes,
            unrelated,
        ));
        assert_eq!(bytes, 0);
        assert!(credit_exact_retirement(
            &displaced,
            &mut completed,
            &mut bytes,
            native_first,
        ));
        assert!(!completed.iter().all(|bit| *bit));
        assert!(!credit_exact_retirement(
            &displaced,
            &mut completed,
            &mut bytes,
            native_first,
        ));
        assert_eq!(bytes, 64);
        assert!(credit_exact_retirement(
            &displaced,
            &mut completed,
            &mut bytes,
            native_second,
        ));
        assert!(completed.iter().all(|bit| *bit));
        assert_eq!(bytes, 128);
    }
}
