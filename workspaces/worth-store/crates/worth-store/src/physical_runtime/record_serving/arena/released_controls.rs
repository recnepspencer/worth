//! Exact, attempt-bound arena claims acquired before any release control can
//! have a durable effect: Manifest, Reservation and Descriptor, plus the
//! replacement directory frame when the drop invalidates the directory
//! watermark. The claims are runtime exclusions, not a durable entitlement
//! after a process crash.

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
    /// The Drop member's replacement directory frame, published in the same
    /// batch as the descriptor.
    directory: Option<(u64, ArenaReservation)>,
}

const CONTROL_CLAIMS: usize = 4;

impl ReleasedControlArenaReservations {
    pub(in crate::physical_runtime::record_serving) fn reserve(
        owner: &SharedArenaAllocationOwner,
        attempt: [u8; 16],
        format: PhysicalRecordFormatDeclaration,
        manifest_encoded_bytes: u64,
        directory_encoded_bytes: Option<u64>,
    ) -> Result<Self, ArenaAllocationDenial> {
        if attempt == [0; 16] {
            return Err(ArenaAllocationDenial::InvalidGeometry);
        }
        let encoded_bytes = [
            Some(manifest_encoded_bytes),
            Some(
                u64::try_from(OriginalDropReservedV1::encoded_frame_bytes())
                    .map_err(|_| ArenaAllocationDenial::InvalidGeometry)?,
            ),
            Some(
                u64::try_from(BlobReclaimDescriptorV3::encoded_frame_bytes())
                    .map_err(|_| ArenaAllocationDenial::InvalidGeometry)?,
            ),
            directory_encoded_bytes,
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
        let mut ranges = [None; CONTROL_CLAIMS];
        for (range, encoded) in ranges.iter_mut().zip(encoded_bytes) {
            let Some(encoded) = encoded else { continue };
            if encoded == 0 {
                return Err(ArenaAllocationDenial::InvalidGeometry);
            }
            let chunks = u32::try_from(encoded.div_ceil(u64::from(chunk_payload_capacity)))
                .map_err(|_| ArenaAllocationDenial::InvalidGeometry)?;
            *range = Some(
                layout
                    .allocated_bytes(chunks)
                    .ok_or(ArenaAllocationDenial::InvalidGeometry)?,
            );
        }

        // One owner lock prevents an unrelated reservation from consuming a
        // later control's range budget between the acquisitions. A failed
        // acquisition cancels raw tokens here, before any claim is exposed.
        let mut acquired: [Option<(u64, ExtentArenaRange)>; CONTROL_CLAIMS] =
            [None; CONTROL_CLAIMS];
        for (index, bytes) in ranges.into_iter().enumerate() {
            let Some(bytes) = bytes else { continue };
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
        let [manifest, reservation, descriptor, directory] = acquired;
        let claim = |claim: Option<_>| claim.expect("every requested control range was reserved");
        let directory = directory_encoded_bytes
            .zip(directory)
            .map(|(bytes, claim)| {
                let (token, range) = claim;
                (bytes, ArenaReservation::from_reserved(owner, token, range))
            });
        let placement = |kind, index: usize, claim| {
            ReleasedControlArenaPlacement::new(
                owner,
                attempt,
                kind,
                encoded_bytes[index].expect("fixed control frames are sized"),
                claim,
            )
        };
        let mut descriptor = placement(BlobRecordKind::ReclaimDescriptorV3, 2, claim(descriptor));
        descriptor.directory = directory;
        Ok(Self {
            manifest: placement(BlobRecordKind::DropSetManifestV3, 0, claim(manifest)),
            reservation: placement(BlobRecordKind::OriginalDropReserved, 1, claim(reservation)),
            descriptor,
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
            directory: None,
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

    #[cfg(test)]
    pub(in crate::physical_runtime) fn encoded_bytes(&self) -> u64 {
        self.encoded_bytes
    }

    #[cfg(test)]
    pub(in crate::physical_runtime::record_serving) fn reservation(&self) -> &ArenaReservation {
        &self.reservation
    }

    /// The replacement directory frame length this placement was sized for.
    pub(in crate::physical_runtime) fn directory_encoded_bytes(&self) -> Option<u64> {
        self.directory.as_ref().map(|(bytes, _)| *bytes)
    }

    /// Batch extents claimed in record order: the control frame, then the
    /// replacement directory frame when one was reserved.
    pub(in crate::physical_runtime::record_serving) fn claim_count(&self) -> usize {
        1 + usize::from(self.directory.is_some())
    }

    pub(in crate::physical_runtime::record_serving) fn claim(
        &self,
        index: usize,
    ) -> Option<(u64, &ArenaReservation)> {
        match index {
            0 => Some((self.encoded_bytes, &self.reservation)),
            1 => self
                .directory
                .as_ref()
                .map(|(bytes, reservation)| (*bytes, reservation)),
            _ => None,
        }
    }

    pub(in crate::physical_runtime::record_serving) fn into_reservations(
        self,
    ) -> (ArenaReservation, Option<ArenaReservation>) {
        (
            self.reservation,
            self.directory.map(|(_, reservation)| reservation),
        )
    }
}

#[cfg(test)]
#[path = "released_controls/tests.rs"]
mod tests;
