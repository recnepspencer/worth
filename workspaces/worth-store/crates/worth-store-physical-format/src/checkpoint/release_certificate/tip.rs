use crate::{OriginalDropReservationRequestV1, PersistedRecordIdentity};

use super::{
    read_record, write_record, Cursor, ReleaseCheckpointCertificateDenial,
    ReleasedDropWalFateWitnessV1,
};

pub const RELEASE_CHECKPOINT_TIP_WIRE_BYTES: usize = 24 + 32 + 24 + 32 + 80 + 80 + 8 + 32;

/// The last selected released drop's exact physical provenance. A successor
/// checkpoint carries this even when its predecessor's Batch was folded and
/// the predecessor checkpoint or WAL was reclaimed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleasedDropTipProvenanceV1 {
    descriptor_record: PersistedRecordIdentity,
    descriptor_frame_sha256: [u8; 32],
    reservation_record: PersistedRecordIdentity,
    reservation_frame_sha256: [u8; 32],
    request: OriginalDropReservationRequestV1,
    fate: ReleasedDropWalFateWitnessV1,
    candidate_root_generation: u64,
    candidate_root_sha256: [u8; 32],
}

impl ReleasedDropTipProvenanceV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        descriptor_record: PersistedRecordIdentity,
        descriptor_frame_sha256: [u8; 32],
        reservation_record: PersistedRecordIdentity,
        reservation_frame_sha256: [u8; 32],
        request: OriginalDropReservationRequestV1,
        fate: ReleasedDropWalFateWitnessV1,
        candidate_root_generation: u64,
        candidate_root_sha256: [u8; 32],
    ) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        if descriptor_record == reservation_record
            || candidate_root_generation == 0
            || [
                descriptor_frame_sha256,
                reservation_frame_sha256,
                candidate_root_sha256,
            ]
            .contains(&[0; 32])
        {
            return Err(ReleaseCheckpointCertificateDenial::InvalidBinding);
        }
        Ok(Self {
            descriptor_record,
            descriptor_frame_sha256,
            reservation_record,
            reservation_frame_sha256,
            request,
            fate,
            candidate_root_generation,
            candidate_root_sha256,
        })
    }

    pub(super) fn write(self, out: &mut Vec<u8>) {
        write_record(out, self.descriptor_record);
        out.extend_from_slice(&self.descriptor_frame_sha256);
        write_record(out, self.reservation_record);
        out.extend_from_slice(&self.reservation_frame_sha256);
        out.extend_from_slice(&self.request.idempotency());
        out.extend_from_slice(&self.request.fingerprint());
        out.extend_from_slice(&self.request.lease_issuance_generation().to_le_bytes());
        out.extend_from_slice(&self.request.lease_expiry_generation().to_le_bytes());
        self.fate.write(out);
        out.extend_from_slice(&self.candidate_root_generation.to_le_bytes());
        out.extend_from_slice(&self.candidate_root_sha256);
    }

    pub(super) fn read(
        cursor: &mut Cursor<'_>,
    ) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        let descriptor_record = read_record(cursor)?;
        let descriptor_frame_sha256 = cursor.take()?;
        let reservation_record = read_record(cursor)?;
        let reservation_frame_sha256 = cursor.take()?;
        let request = OriginalDropReservationRequestV1::new(
            cursor.take()?,
            cursor.take()?,
            u64::from_le_bytes(cursor.take()?),
            u64::from_le_bytes(cursor.take()?),
        )
        .map_err(|_| ReleaseCheckpointCertificateDenial::Malformed)?;
        let fate = ReleasedDropWalFateWitnessV1::read(cursor)?;
        Self::new(
            descriptor_record,
            descriptor_frame_sha256,
            reservation_record,
            reservation_frame_sha256,
            request,
            fate,
            u64::from_le_bytes(cursor.take()?),
            cursor.take()?,
        )
    }

    pub const fn descriptor_record(self) -> PersistedRecordIdentity {
        self.descriptor_record
    }
    pub const fn descriptor_frame_sha256(self) -> [u8; 32] {
        self.descriptor_frame_sha256
    }
    pub const fn reservation_record(self) -> PersistedRecordIdentity {
        self.reservation_record
    }
    pub const fn reservation_frame_sha256(self) -> [u8; 32] {
        self.reservation_frame_sha256
    }
    pub const fn request(self) -> OriginalDropReservationRequestV1 {
        self.request
    }
    pub const fn fate(self) -> ReleasedDropWalFateWitnessV1 {
        self.fate
    }
    pub const fn candidate_root_generation(self) -> u64 {
        self.candidate_root_generation
    }
    pub const fn candidate_root_sha256(self) -> [u8; 32] {
        self.candidate_root_sha256
    }
}
