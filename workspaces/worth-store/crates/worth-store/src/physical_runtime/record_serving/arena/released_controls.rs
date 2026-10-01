//! Three exact, attempt-bound arena claims acquired before any release control
//! can have a durable effect. The claims are runtime exclusions, not a durable
//! entitlement after a process crash.

use super::{ArenaAllocationDenial, ArenaReservation, SharedArenaAllocationOwner};
use worth_store_physical_format::{
    BlobReclaimDescriptorV3, BlobRecordKind, ExtentArenaFrameLayout, ExtentArenaRange,
    OriginalDropReservedV1, PhysicalRecordFormatDeclaration, PhysicalTierClass,
    DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};

pub(in crate::physical_runtime) struct ReleasedControlArenaReservations {
    manifest: ReleasedControlArenaPlacement,
    reservation: ReleasedControlArenaPlacement,
    descriptor: ReleasedControlArenaPlacement,
}

pub(in crate::physical_runtime) struct ReleasedControlArenaPlacement {
    attempt: [u8; 16],
    kind: BlobRecordKind,
    encoded_bytes: u64,
    reservation: ArenaReservation,
}

impl ReleasedControlArenaReservations {
    pub(in crate::physical_runtime::record_serving) fn reserve(
        owner: &SharedArenaAllocationOwner,
        attempt: [u8; 16],
        format: PhysicalRecordFormatDeclaration,
        manifest_encoded_bytes: u64,
    ) -> Result<Self, ArenaAllocationDenial> {
        if attempt == [0; 16] {
            return Err(ArenaAllocationDenial::InvalidGeometry);
        }
        let encoded_bytes = [
            manifest_encoded_bytes,
            u64::try_from(OriginalDropReservedV1::encoded_frame_bytes())
                .map_err(|_| ArenaAllocationDenial::InvalidGeometry)?,
            u64::try_from(BlobReclaimDescriptorV3::encoded_frame_bytes())
                .map_err(|_| ArenaAllocationDenial::InvalidGeometry)?,
        ];
        let mut allocation = owner.lock().unwrap_or_else(|e| e.into_inner());
        let layout = ExtentArenaFrameLayout::new(format, allocation.alignment())
            .ok_or(ArenaAllocationDenial::InvalidGeometry)?;
        let frame_header =
            u32::try_from(DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES)
                .map_err(|_| ArenaAllocationDenial::InvalidGeometry)?;
        let chunk_payload_capacity = format
            .page_size()
            .bytes()
            .checked_sub(frame_header)
            .filter(|capacity| *capacity != 0)
            .ok_or(ArenaAllocationDenial::InvalidGeometry)?;
        let mut ranges = [0_u64; 3];
        for (range, encoded) in ranges.iter_mut().zip(encoded_bytes) {
            if encoded == 0 {
                return Err(ArenaAllocationDenial::InvalidGeometry);
            }
            let chunks = u32::try_from(encoded.div_ceil(u64::from(chunk_payload_capacity)))
                .map_err(|_| ArenaAllocationDenial::InvalidGeometry)?;
            *range = layout
                .allocated_bytes(chunks)
                .ok_or(ArenaAllocationDenial::InvalidGeometry)?;
        }

        // One owner lock prevents an unrelated reservation from consuming the
        // later control's range budget between the three acquisitions. A failed
        // acquisition cancels raw tokens here, before any claim is exposed.
        let mut acquired: [Option<(u64, ExtentArenaRange)>; 3] = [None; 3];
        for (index, bytes) in ranges.into_iter().enumerate() {
            match allocation.reserve_in_tier(bytes, PhysicalTierClass::Primary) {
                Ok(claim) => acquired[index] = Some(claim),
                Err(cause) => {
                    for (token, _) in acquired.iter().rev().flatten() {
                        allocation
                            .cancel(*token)
                            .expect("a pre-effect arena claim remains owned until cancellation");
                    }
                    return Err(cause);
                }
            }
        }
        drop(allocation);
        let [manifest, reservation, descriptor] =
            acquired.map(|claim| claim.expect("all three control ranges were reserved"));
        Ok(Self {
            manifest: ReleasedControlArenaPlacement::new(
                owner,
                attempt,
                BlobRecordKind::DropSetManifestV3,
                encoded_bytes[0],
                manifest,
            ),
            reservation: ReleasedControlArenaPlacement::new(
                owner,
                attempt,
                BlobRecordKind::OriginalDropReserved,
                encoded_bytes[1],
                reservation,
            ),
            descriptor: ReleasedControlArenaPlacement::new(
                owner,
                attempt,
                BlobRecordKind::ReclaimDescriptorV3,
                encoded_bytes[2],
                descriptor,
            ),
        })
    }

    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        ReleasedControlArenaPlacement,
        ReleasedControlArenaPlacement,
        ReleasedControlArenaPlacement,
    ) {
        (self.manifest, self.reservation, self.descriptor)
    }
}

impl ReleasedControlArenaPlacement {
    fn new(
        owner: &SharedArenaAllocationOwner,
        attempt: [u8; 16],
        kind: BlobRecordKind,
        encoded_bytes: u64,
        (token, range): (u64, ExtentArenaRange),
    ) -> Self {
        Self {
            attempt,
            kind,
            encoded_bytes,
            reservation: ArenaReservation::from_reserved(owner, token, range),
        }
    }

    pub(in crate::physical_runtime) fn matches(
        &self,
        attempt: [u8; 16],
        kind: BlobRecordKind,
        encoded_bytes: u64,
    ) -> bool {
        self.attempt == attempt && self.kind == kind && self.encoded_bytes == encoded_bytes
    }

    pub(in crate::physical_runtime) fn encoded_bytes(&self) -> u64 {
        self.encoded_bytes
    }

    pub(in crate::physical_runtime::record_serving) fn reservation(&self) -> &ArenaReservation {
        &self.reservation
    }

    pub(in crate::physical_runtime::record_serving) fn into_reservation(self) -> ArenaReservation {
        self.reservation
    }
}

#[cfg(test)]
#[path = "released_controls/tests.rs"]
mod tests;
