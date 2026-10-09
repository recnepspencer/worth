//! Native admission follows C4's staged co-live listing storage contract.

use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_backend::{
    ArtifactTreeDirectoryEntry, ArtifactTreeListingAllocationBoundary as Boundary,
    ArtifactTreeListingStorageChange as Change,
};

use super::super::{
    PhysicalRecoveryObservationAllocationDenial as Denial, PhysicalRecoveryReadAllocation,
};
use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial;

#[derive(Default)]
pub(super) struct NativeWalListingBacking {
    grant: Option<OperationAllocationGrant>,
}

impl NativeWalListingBacking {
    pub(super) fn change(
        &mut self,
        window: &PhysicalRecoveryReadAllocation<'_>,
        change: Change,
    ) -> Result<(), Denial> {
        match change {
            Change::Admit {
                boundary,
                required_bytes,
            } => {
                if required_bytes <= self.charged_bytes() {
                    return Ok(());
                }
                let result = match &mut self.grant {
                    Some(grant) => grant
                        .try_resize(required_bytes)
                        .map_err(|cause| window.map_allocation_denial(cause)),
                    None => window
                        .reserve_owned(required_bytes)
                        .map(|grant| self.grant = grant),
                };
                result.map_err(|cause| Denial::ListingResidency { boundary, cause })
            }
            Change::Settle { retained_bytes } => {
                self.settle_after_disposal(retained_bytes);
                Ok(())
            }
        }
    }

    pub(super) fn allocate_roster(
        &self,
        count: usize,
    ) -> Result<Vec<ArtifactTreeDirectoryEntry>, Denial> {
        let requested = count
            .checked_mul(std::mem::size_of::<ArtifactTreeDirectoryEntry>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .expect("C4 checked the admitted listing layout");
        assert!(
            requested <= self.charged_bytes(),
            "listing storage must precede construction"
        );
        let mut roster = Vec::new();
        roster
            .try_reserve_exact(count)
            .map_err(|cause| Denial::ListingResidency {
                boundary: Boundary::EntryRoster,
                cause: PhysicalRecoveryRejoinResidentDenial::Allocation { requested, cause },
            })?;
        let actual =
            roster.capacity() as u64 * std::mem::size_of::<ArtifactTreeDirectoryEntry>() as u64;
        if actual != requested {
            return Err(Denial::AllocatorExceededReservation { requested, actual });
        }
        Ok(roster)
    }

    pub(super) fn settle_after_disposal(&mut self, retained_bytes: u64) {
        assert!(
            retained_bytes <= self.charged_bytes(),
            "settlement cannot authorize new allocation"
        );
        if retained_bytes == 0 {
            self.grant = None;
        } else {
            self.grant
                .as_mut()
                .expect("positive retained listing was admitted")
                .try_resize(retained_bytes)
                .expect("private listing backing has no named live uses");
        }
    }

    pub(super) fn into_names(
        mut self,
        artifacts: &[super::ObservedWalArtifact],
    ) -> Option<OperationAllocationGrant> {
        let names = artifacts
            .iter()
            .try_fold(0_u64, |bytes, artifact| {
                bytes.checked_add(artifact.name_heap_bytes() as u64)
            })
            .expect("retained names are bounded by admitted listing storage");
        // C4's listing IntoIter and provider have been disposed before returning.
        self.settle_after_disposal(names);
        self.grant
    }

    fn charged_bytes(&self) -> u64 {
        self.grant
            .as_ref()
            .map_or(0, OperationAllocationGrant::bytes)
    }
}
