use crate::{OriginalDropReservationRequestV1, PersistedRecordIdentity, ReleasedDropPredecessorV1};

use super::{
    read_checkpoint, read_record, Cursor, PhysicalCheckpointIdentity,
    ReleaseCheckpointCertificateDenial, ReleasedDropTipProvenanceV1, ReleasedDropWalFateWitnessV1,
    BATCH_KIND, PREFIX_BYTES,
};

pub const RELEASE_CHECKPOINT_BATCH_WIRE_BYTES: usize =
    PREFIX_BYTES + 24 + 8 + 32 + 2 + 24 + 32 + 32 + 24 + 32 + 80 + 80 + 8 + 32 + 56 + 8 + 32 + 1;

/// One selected V3 batch folded into a checkpoint. This is a format claim,
/// not release authority: recovery joins its exact routes, WAL and fate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseCheckpointBatchV1 {
    checkpoint: PhysicalCheckpointIdentity,
    root_generation: u64,
    root_sha256: [u8; 32],
    ordinal: u16,
    descriptor_record: PersistedRecordIdentity,
    descriptor_frame_sha256: [u8; 32],
    custody_digest: [u8; 32],
    reservation_record: PersistedRecordIdentity,
    reservation_frame_sha256: [u8; 32],
    request: OriginalDropReservationRequestV1,
    fate: ReleasedDropWalFateWitnessV1,
    candidate_root_generation: u64,
    candidate_root_sha256: [u8; 32],
    predecessor: Option<ReleasedDropPredecessorV1>,
    cumulative_dropped: u64,
    cumulative_digest: [u8; 32],
    terminal: bool,
}

impl ReleaseCheckpointBatchV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        checkpoint: PhysicalCheckpointIdentity,
        root_generation: u64,
        root_sha256: [u8; 32],
        ordinal: u16,
        descriptor_record: PersistedRecordIdentity,
        descriptor_frame_sha256: [u8; 32],
        custody_digest: [u8; 32],
        reservation_record: PersistedRecordIdentity,
        reservation_frame_sha256: [u8; 32],
        request: OriginalDropReservationRequestV1,
        fate: ReleasedDropWalFateWitnessV1,
        candidate_root_generation: u64,
        candidate_root_sha256: [u8; 32],
        predecessor: Option<ReleasedDropPredecessorV1>,
        cumulative_dropped: u64,
        cumulative_digest: [u8; 32],
        terminal: bool,
    ) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        if root_generation == 0
            || root_generation < candidate_root_generation
            || candidate_root_generation == 0
            || ordinal >= 64
            || cumulative_dropped == 0
            || [
                root_sha256,
                descriptor_frame_sha256,
                custody_digest,
                reservation_frame_sha256,
                candidate_root_sha256,
                cumulative_digest,
            ]
            .contains(&[0; 32])
            || descriptor_record == reservation_record
            || predecessor.is_some_and(|prior| prior.descriptor_record() == descriptor_record)
        {
            return Err(ReleaseCheckpointCertificateDenial::InvalidBinding);
        }
        Ok(Self {
            checkpoint,
            root_generation,
            root_sha256,
            ordinal,
            descriptor_record,
            descriptor_frame_sha256,
            custody_digest,
            reservation_record,
            reservation_frame_sha256,
            request,
            fate,
            candidate_root_generation,
            candidate_root_sha256,
            predecessor,
            cumulative_dropped,
            cumulative_digest,
            terminal,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        self.encode_in_reserved(Vec::with_capacity(RELEASE_CHECKPOINT_BATCH_WIRE_BYTES))
            .expect("fixed release batch wire capacity")
    }

    /// Reuses caller-admitted backing; insufficient capacity performs no write.
    pub fn encode_in_reserved(self, mut out: Vec<u8>) -> Option<Vec<u8>> {
        if out.capacity() < RELEASE_CHECKPOINT_BATCH_WIRE_BYTES {
            return None;
        }
        out.clear();
        out.extend_from_slice(&self.encode_fixed());
        Some(out)
    }

    pub fn encode_fixed(self) -> [u8; RELEASE_CHECKPOINT_BATCH_WIRE_BYTES] {
        let mut out = [0; RELEASE_CHECKPOINT_BATCH_WIRE_BYTES];
        let mut checkpoint = [0; 24];
        super::encode_identity(&mut checkpoint, self.checkpoint);
        let mut cursor = 0;
        let mut write = |value: &[u8]| {
            out[cursor..cursor + value.len()].copy_from_slice(value);
            cursor += value.len();
        };
        write(&(super::DOMAIN.len() as u64).to_le_bytes());
        write(super::DOMAIN);
        write(&[super::VERSION, BATCH_KIND]);
        write(&checkpoint);
        write(&self.root_generation.to_le_bytes());
        write(&self.root_sha256);
        write(&self.ordinal.to_le_bytes());
        write(&self.descriptor_record.allocation_epoch());
        write(&self.descriptor_record.ordinal().to_le_bytes());
        write(&self.descriptor_frame_sha256);
        write(&self.custody_digest);
        write(&self.reservation_record.allocation_epoch());
        write(&self.reservation_record.ordinal().to_le_bytes());
        write(&self.reservation_frame_sha256);
        write(&self.request.idempotency());
        write(&self.request.fingerprint());
        write(&self.request.lease_issuance_generation().to_le_bytes());
        write(&self.request.lease_expiry_generation().to_le_bytes());
        write(&self.fate.lsn_start().to_le_bytes());
        write(&self.fate.lsn_end_exclusive().to_le_bytes());
        write(&self.fate.identity_digest());
        write(&self.fate.payload_digest());
        write(&self.candidate_root_generation.to_le_bytes());
        write(&self.candidate_root_sha256);
        if let Some(prior) = self.predecessor {
            write(&prior.descriptor_record().allocation_epoch());
            write(&prior.descriptor_record().ordinal().to_le_bytes());
            write(&prior.descriptor_frame_sha256());
        } else {
            write(&[0; 56]);
        }
        write(&self.cumulative_dropped.to_le_bytes());
        write(&self.cumulative_digest);
        write(&[u8::from(self.terminal)]);
        debug_assert_eq!(cursor, RELEASE_CHECKPOINT_BATCH_WIRE_BYTES);
        out
    }

    pub(super) fn read(
        cursor: &mut Cursor<'_>,
    ) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        let checkpoint = read_checkpoint(cursor)?;
        let root_generation = u64::from_le_bytes(cursor.take()?);
        let root_sha256 = cursor.take()?;
        let ordinal = u16::from_le_bytes(cursor.take()?);
        let descriptor_record = read_record(cursor)?;
        let descriptor_frame_sha256 = cursor.take()?;
        let custody_digest = cursor.take()?;
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
        let candidate_root_generation = u64::from_le_bytes(cursor.take()?);
        let candidate_root_sha256 = cursor.take()?;
        let prior_bytes = cursor.take::<56>()?;
        let predecessor = if prior_bytes == [0; 56] {
            None
        } else {
            let record = PersistedRecordIdentity::new(
                prior_bytes[..16].try_into().unwrap(),
                u64::from_le_bytes(prior_bytes[16..24].try_into().unwrap()),
            )
            .ok_or(ReleaseCheckpointCertificateDenial::Malformed)?;
            Some(
                ReleasedDropPredecessorV1::new(record, prior_bytes[24..56].try_into().unwrap())
                    .map_err(|_| ReleaseCheckpointCertificateDenial::Malformed)?,
            )
        };
        let cumulative_dropped = u64::from_le_bytes(cursor.take()?);
        let cumulative_digest = cursor.take()?;
        let terminal = match cursor.take::<1>()?[0] {
            0 => false,
            1 => true,
            _ => return Err(ReleaseCheckpointCertificateDenial::Malformed),
        };
        Self::new(
            checkpoint,
            root_generation,
            root_sha256,
            ordinal,
            descriptor_record,
            descriptor_frame_sha256,
            custody_digest,
            reservation_record,
            reservation_frame_sha256,
            request,
            fate,
            candidate_root_generation,
            candidate_root_sha256,
            predecessor,
            cumulative_dropped,
            cumulative_digest,
            terminal,
        )
    }

    pub const fn checkpoint(self) -> PhysicalCheckpointIdentity {
        self.checkpoint
    }
    pub const fn root_generation(self) -> u64 {
        self.root_generation
    }
    pub const fn root_sha256(self) -> [u8; 32] {
        self.root_sha256
    }
    pub const fn ordinal(self) -> u16 {
        self.ordinal
    }
    pub const fn descriptor_record(self) -> PersistedRecordIdentity {
        self.descriptor_record
    }
    pub const fn descriptor_frame_sha256(self) -> [u8; 32] {
        self.descriptor_frame_sha256
    }
    pub const fn custody_digest(self) -> [u8; 32] {
        self.custody_digest
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
    pub const fn predecessor(self) -> Option<ReleasedDropPredecessorV1> {
        self.predecessor
    }
    pub const fn cumulative_dropped(self) -> u64 {
        self.cumulative_dropped
    }
    pub const fn cumulative_digest(self) -> [u8; 32] {
        self.cumulative_digest
    }
    pub const fn terminal(self) -> bool {
        self.terminal
    }

    pub fn tip_provenance(
        self,
    ) -> Result<ReleasedDropTipProvenanceV1, ReleaseCheckpointCertificateDenial> {
        ReleasedDropTipProvenanceV1::new(
            self.descriptor_record,
            self.descriptor_frame_sha256,
            self.reservation_record,
            self.reservation_frame_sha256,
            self.request,
            self.fate,
            self.candidate_root_generation,
            self.candidate_root_sha256,
        )
    }
}
