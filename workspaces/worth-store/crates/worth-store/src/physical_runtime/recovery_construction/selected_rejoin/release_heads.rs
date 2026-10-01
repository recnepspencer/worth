//! Independent bounded walk of the selected head tree. Certificate digests
//! are checked against actual rooted blocks, not substituted for membership.

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadEntryV1,
};
use worth_store_physical_integrity::{
    walk_release_custody_head_with_port, ReleaseCustodyHeadWalkDenial,
    ReleaseCustodyHeadWalkLimitsV1,
};

use super::control_frames::{SelectedArtifactSlice, SelectedControlMediaFingerprint};
use super::resident::{
    discovery_allocation_denial, PhysicalRecoveryRejoinResidentDenial, StoreRejoinResidentLedger,
};
use super::SelectedMediaRejoinDenial as Denial;
use crate::physical_runtime::{
    durability::ReleaseHeadCapacityCharge, PhysicalRecoveryAllocationAdmission,
};

mod controls;
mod port;
pub(super) use controls::{
    observe_controls, observe_controls_on_routes_with_resident, ObservedReleaseHeadControls,
};
#[cfg(test)]
#[path = "release_heads/tests.rs"]
mod tests;

pub(super) struct ObservedReleaseHeads {
    entries: Vec<ReleaseCustodyHeadEntryV1>,
    slices: Vec<SelectedArtifactSlice>,
    count: u64,
    digest: [u8; 32],
}

impl ObservedReleaseHeads {
    pub(super) fn entries(&self) -> &[ReleaseCustodyHeadEntryV1] {
        &self.entries
    }
    pub(super) fn count(&self) -> u64 {
        self.count
    }
    pub(super) fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub(super) fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.entries.capacity())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<ReleaseCustodyHeadEntryV1>()).ok()?)?
            .checked_add(
                u64::try_from(self.slices.capacity()).ok()?.checked_mul(
                    u64::try_from(std::mem::size_of::<SelectedArtifactSlice>()).ok()?,
                )?,
            )
    }

    pub(super) fn same_bytes(&self, other: &Self) -> bool {
        self.entries == other.entries
            && self.slices == other.slices
            && self.count == other.count
            && self.digest == other.digest
    }

    pub(super) fn into_fingerprint(self) -> SelectedControlMediaFingerprint {
        SelectedControlMediaFingerprint::observed(self.slices)
    }

    pub(super) fn into_fingerprint_with_resident(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<SelectedControlMediaFingerprint, Denial> {
        let entries_bytes = resident
            .vector_bytes(&self.entries)
            .map_err(Denial::Resident)?;
        let Self {
            entries, slices, ..
        } = self;
        drop(entries);
        resident.release(entries_bytes);
        Ok(SelectedControlMediaFingerprint::observed(slices))
    }
}

/// The older pending V14 caller retains its local bound. V2 never uses this
/// constructor: it consumes the coordination-carried resident ledger.
pub(super) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    recovery_allocation: PhysicalRecoveryAllocationAdmission,
    expected_count: u64,
    expected_digest: [u8; 32],
    maximum_resident_bytes: u64,
) -> Result<ObservedReleaseHeads, Denial> {
    let mut resident = StoreRejoinResidentLedger::pending_head_walk_window(
        recovery_allocation,
        maximum_resident_bytes,
    )
    .map_err(Denial::Resident)?;
    observe_with_resident(
        discovery,
        root,
        format,
        recovery_allocation,
        expected_count,
        expected_digest,
        &mut resident,
    )
}

pub(super) fn observe_with_resident(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    recovery_allocation: PhysicalRecoveryAllocationAdmission,
    expected_count: u64,
    expected_digest: [u8; 32],
    resident: &mut StoreRejoinResidentLedger,
) -> Result<ObservedReleaseHeads, Denial> {
    if discovery.store_identity() != recovery_allocation.store_identity() {
        return Err(Denial::RootBinding);
    }
    observe_with_read(
        root,
        format,
        expected_count,
        expected_digest,
        recovery_allocation.byte_limit(),
        resident,
        |reference, remaining, resident| {
            let artifact = RecordArtifactFile::ReleaseCustodyHeadBlock {
                generation: reference.generation(),
                block: reference.block(),
            };
            discovery
                .read_record_artifact_with_allocator(artifact, remaining, |length| {
                    resident.reserve_bytes(length)
                })
                .map_err(discovery_allocation_denial)?
                .into_bytes()
                .ok_or(Denial::MissingFrame)
        },
    )
}

/// The read callback returns a frame charged to `resident`. Production
/// reads actual selected media; tests only observe this pre-read boundary.
#[allow(clippy::too_many_arguments)]
fn observe_with_read(
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    expected_count: u64,
    expected_digest: [u8; 32],
    owner_byte_limit: u64,
    resident: &mut StoreRejoinResidentLedger,
    read: impl FnMut(
        ReleaseCustodyHeadBlockReferenceV1,
        u64,
        &mut StoreRejoinResidentLedger,
    ) -> Result<Vec<u8>, Denial>,
) -> Result<ObservedReleaseHeads, Denial> {
    let page_bytes = u64::from(format.page_size().bytes());
    let closure =
        ReleaseHeadCapacityCharge::selected_roster_closure_bytes(expected_count, page_bytes)
            .ok_or(Denial::BoundExceeded)?;
    if closure > owner_byte_limit {
        return Err(Denial::BoundExceeded);
    }
    let max_nodes = if expected_count == 0 {
        1
    } else {
        expected_count
            .checked_mul(2)
            .and_then(|nodes| nodes.checked_add(16))
            .ok_or(Denial::BoundExceeded)?
    };
    let max_frame_bytes = max_nodes
        .checked_mul(page_bytes)
        .ok_or(Denial::BoundExceeded)?
        .min(owner_byte_limit);
    let entry_capacity = usize::try_from(expected_count).map_err(|_| Denial::BoundExceeded)?;
    let slice_capacity = usize::try_from(max_nodes).map_err(|_| Denial::BoundExceeded)?;
    let requested_entries = expected_count
        .checked_mul(std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64)
        .ok_or(Denial::BoundExceeded)?;
    let requested_slices = max_nodes
        .checked_mul(std::mem::size_of::<SelectedArtifactSlice>() as u64)
        .ok_or(Denial::BoundExceeded)?;
    let minimum_walk = if root.release_custody_head_root().is_some() {
        ReleaseCustodyHeadWalkLimitsV1::root_resident_preflight_bytes(format, max_nodes)
            .ok_or(Denial::BoundExceeded)?
    } else {
        1
    };
    let required = requested_entries
        .checked_add(requested_slices)
        .and_then(|bytes| bytes.checked_add(minimum_walk))
        .ok_or(Denial::BoundExceeded)?;
    resident.transient(required).map_err(Denial::Resident)?;
    let entries = resident
        .reserve_vec(entry_capacity)
        .map_err(Denial::Resident)?;
    let slices_and_walk = requested_slices
        .checked_add(minimum_walk)
        .ok_or(Denial::BoundExceeded)?;
    // The allocator may reserve more entry capacity than requested. Keep
    // the remaining output and first-root walk live in the same preflight.
    resident
        .transient(slices_and_walk)
        .map_err(Denial::Resident)?;
    let slices = resident
        .reserve_vec(slice_capacity)
        .map_err(Denial::Resident)?;
    let walker_ceiling = resident.remaining();
    resident.transient(minimum_walk).map_err(Denial::Resident)?;
    let limits = ReleaseCustodyHeadWalkLimitsV1::new(
        max_nodes,
        expected_count,
        max_frame_bytes,
        walker_ceiling,
        64,
    )
    .ok_or(Denial::BoundExceeded)?;
    let walker_base = resident.used();
    let walker_admitted = walker_base
        .checked_add(walker_ceiling)
        .expect("remaining derives from this ledger's maximum");
    let mut port = port::StoreHeadWalkPort::new(resident, read, entries, slices);
    let walk =
        walk_release_custody_head_with_port(root, format, limits, &mut port).map_err(|denial| {
            match denial {
                ReleaseCustodyHeadWalkDenial::Read(denial)
                | ReleaseCustodyHeadWalkDenial::Visit(denial)
                | ReleaseCustodyHeadWalkDenial::Storage(denial) => denial,
                ReleaseCustodyHeadWalkDenial::ResidentBoundExceeded { required, admitted } => {
                    match (
                        walker_base.checked_add(required),
                        walker_base.checked_add(admitted),
                    ) {
                        (Some(required), Some(admitted)) => {
                            Denial::Resident(PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                                required,
                                admitted,
                            })
                        }
                        _ => Denial::Resident(PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
                            admitted: walker_admitted,
                        }),
                    }
                }
                ReleaseCustodyHeadWalkDenial::BoundExceeded => Denial::BoundExceeded,
                _ => Denial::CertificateRoster,
            }
        })?;
    if walk.entry_count() != expected_count || walk.roster_digest() != expected_digest {
        return Err(Denial::CertificateRoster);
    }
    let (entries, slices) = port.into_outputs();
    Ok(ObservedReleaseHeads {
        entries,
        slices,
        count: walk.entry_count(),
        digest: walk.roster_digest(),
    })
}
