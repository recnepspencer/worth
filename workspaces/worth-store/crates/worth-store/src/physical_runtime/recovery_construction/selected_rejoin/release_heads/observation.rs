//! Capacity preparation, canonical walk and allocation-safe terminal handling.

use super::{
    backing::{slot_bytes, HeadWalkBacking},
    storage::HeadWalkStorage,
    Denial, ObservedReleaseHeads, SelectedArtifactSlice, StoreRejoinResidentLedger,
};
use crate::physical_runtime::{
    durability::ReleaseHeadCapacityCharge, PhysicalRecoveryReadAllocation,
    PhysicalRecoveryRejoinResidentDenial,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadEntryV1,
};
use worth_store_physical_integrity::{
    walk_release_custody_head_with_port, ReleaseCustodyHeadWalkDenial,
    ReleaseCustodyHeadWalkLimitsV1,
};

pub(super) fn observe_with_read(
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    expected_count: u64,
    expected_digest: [u8; 32],
    owner_byte_limit: u64,
    window: &PhysicalRecoveryReadAllocation<'_>,
    resident: &mut StoreRejoinResidentLedger,
    read: impl FnMut(
        ReleaseCustodyHeadBlockReferenceV1,
        u64,
        &mut HeadWalkStorage<'_, '_>,
    ) -> Result<Vec<u8>, Denial>,
) -> Result<ObservedReleaseHeads, Denial> {
    let capacity = HeadWalkCapacity::prepare(root, format, expected_count, owner_byte_limit)?;
    resident
        .transient(capacity.initial_bytes)
        .map_err(Denial::Resident)?;
    // Grant precedes every owned vector, and survives the Integrity stack's
    // implicit error drops as well as the port's output-vector disposal.
    let mut backing = HeadWalkBacking::new(window)?;
    backing.prepare(window, capacity.initial_bytes)?;
    let numeric_base = resident.used();
    let result = observe_prepared(
        root,
        format,
        expected_count,
        expected_digest,
        capacity,
        window,
        resident,
        &mut backing,
        read,
    );
    match result {
        Ok((entries, slices, count, digest)) => {
            backing.settle(backing.used())?;
            Ok(ObservedReleaseHeads {
                entries,
                slices,
                count,
                digest,
                backing,
            })
        }
        Err(denial) => {
            // Returning disposed walker scratch and port outputs even when
            // Integrity's error path omitted its success-only discard hooks.
            resident.release(
                resident
                    .used()
                    .checked_sub(numeric_base)
                    .expect("local retained capacity"),
            );
            Err(denial)
        }
    }
}

fn observe_prepared(
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    expected_count: u64,
    expected_digest: [u8; 32],
    capacity: HeadWalkCapacity,
    window: &PhysicalRecoveryReadAllocation<'_>,
    resident: &mut StoreRejoinResidentLedger,
    backing: &mut HeadWalkBacking,
    read: impl FnMut(
        ReleaseCustodyHeadBlockReferenceV1,
        u64,
        &mut HeadWalkStorage<'_, '_>,
    ) -> Result<Vec<u8>, Denial>,
) -> Result<
    (
        Vec<ReleaseCustodyHeadEntryV1>,
        Vec<SelectedArtifactSlice>,
        u64,
        [u8; 32],
    ),
    Denial,
> {
    let mut storage = HeadWalkStorage::new(window, resident, backing);
    let entries = storage.reserve_vec(capacity.entries)?;
    let slices = storage.reserve_vec(capacity.slices)?;
    let walker_ceiling = storage.resident.remaining();
    storage
        .resident
        .transient(capacity.minimum_walk)
        .map_err(Denial::Resident)?;
    let limits = ReleaseCustodyHeadWalkLimitsV1::new(
        capacity.nodes,
        expected_count,
        capacity.max_frame_bytes,
        walker_ceiling,
        64,
    )
    .ok_or(Denial::BoundExceeded)?;
    let walker_base = storage.resident.used();
    let walker_admitted = walker_base
        .checked_add(walker_ceiling)
        .ok_or(Denial::BoundExceeded)?;
    let mut port = super::port::StoreHeadWalkPort::new(storage, read, entries, slices);
    let walk = walk_release_custody_head_with_port(root, format, limits, &mut port)
        .map_err(|denial| walk_denial(denial, walker_base, walker_admitted))?;
    if walk.entry_count() != expected_count || walk.roster_digest() != expected_digest {
        return Err(Denial::CertificateRoster);
    }
    let (entries, slices) = port.into_outputs();
    Ok((entries, slices, walk.entry_count(), walk.roster_digest()))
}

struct HeadWalkCapacity {
    entries: usize,
    slices: usize,
    nodes: u64,
    max_frame_bytes: u64,
    minimum_walk: u64,
    initial_bytes: u64,
}

impl HeadWalkCapacity {
    fn prepare(
        root: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
        count: u64,
        ceiling: u64,
    ) -> Result<Self, Denial> {
        let page = u64::from(format.page_size().bytes());
        let closure = ReleaseHeadCapacityCharge::selected_roster_closure_bytes(count, page)
            .ok_or(Denial::BoundExceeded)?;
        if closure > ceiling {
            return Err(Denial::BoundExceeded);
        }
        let nodes = if count == 0 {
            1
        } else {
            count
                .checked_mul(2)
                .and_then(|n| n.checked_add(16))
                .ok_or(Denial::BoundExceeded)?
        };
        let entries = usize::try_from(count).map_err(|_| Denial::BoundExceeded)?;
        let rooted = root.release_custody_head_root().is_some();
        let slices = if rooted {
            usize::try_from(nodes).map_err(|_| Denial::BoundExceeded)?
        } else {
            0
        };
        let minimum_walk = if rooted {
            ReleaseCustodyHeadWalkLimitsV1::root_resident_preflight_bytes(format, nodes)
                .ok_or(Denial::BoundExceeded)?
        } else {
            0
        };
        let initial_bytes = slot_bytes::<ReleaseCustodyHeadEntryV1>(entries)?
            .checked_add(slot_bytes::<SelectedArtifactSlice>(slices)?)
            .and_then(|bytes| bytes.checked_add(minimum_walk))
            .ok_or(Denial::BoundExceeded)?;
        Ok(Self {
            entries,
            slices,
            nodes,
            max_frame_bytes: nodes
                .checked_mul(page)
                .ok_or(Denial::BoundExceeded)?
                .min(ceiling),
            minimum_walk,
            initial_bytes,
        })
    }
}

fn walk_denial(
    denial: ReleaseCustodyHeadWalkDenial<Denial, Denial>,
    base: u64,
    admitted: u64,
) -> Denial {
    match denial {
        ReleaseCustodyHeadWalkDenial::Read(denial)
        | ReleaseCustodyHeadWalkDenial::Visit(denial)
        | ReleaseCustodyHeadWalkDenial::Storage(denial) => denial,
        ReleaseCustodyHeadWalkDenial::ResidentBoundExceeded {
            required,
            admitted: limit,
        } => match (base.checked_add(required), base.checked_add(limit)) {
            (Some(required), Some(admitted)) => {
                Denial::Resident(PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                    required,
                    admitted,
                })
            }
            _ => Denial::Resident(PhysicalRecoveryRejoinResidentDenial::SizeOverflow { admitted }),
        },
        // Store names one bound until its denial carries the walker's count.
        ReleaseCustodyHeadWalkDenial::BoundExceeded
        | ReleaseCustodyHeadWalkDenial::NodeBound { .. } => Denial::BoundExceeded,
        _ => Denial::CertificateRoster,
    }
}
