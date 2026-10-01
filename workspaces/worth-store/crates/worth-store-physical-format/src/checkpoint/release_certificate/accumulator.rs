use super::{
    read_checkpoint, write_checkpoint, write_prefix, Cursor, PhysicalCheckpointIdentity,
    ReleaseCheckpointCertificateDenial, ReleasedDropTipProvenanceV1, ACCUMULATOR_KIND,
    PREFIX_BYTES, RELEASE_CHECKPOINT_TIP_WIRE_BYTES,
};

pub const RELEASE_CHECKPOINT_ACCUMULATOR_WIRE_BYTES: usize = PREFIX_BYTES
    + 24
    + 8
    + 32
    + 8
    + 32
    + 32
    + 8
    + 32
    + 2
    + 32
    + RELEASE_CHECKPOINT_TIP_WIRE_BYTES
    + 8
    + 32
    + 1;

/// A selected checkpoint's ordered tag-7 batch digest and prior-checkpoint
/// ratchet and exact selected tip provenance. An absent prior has zeroed
/// sequence, root, digest, and cumulative state; reopen never infers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseCheckpointAccumulatorV1 {
    checkpoint: PhysicalCheckpointIdentity,
    root_generation: u64,
    root_sha256: [u8; 32],
    prior_checkpoint_sequence: u64,
    prior_root_sha256: [u8; 32],
    prior_accumulator_digest: [u8; 32],
    prior_cumulative_dropped: u64,
    prior_cumulative_digest: [u8; 32],
    batch_count: u16,
    batch_records_digest: [u8; 32],
    tip: ReleasedDropTipProvenanceV1,
    cumulative_dropped: u64,
    cumulative_digest: [u8; 32],
    terminal: bool,
}

impl ReleaseCheckpointAccumulatorV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        checkpoint: PhysicalCheckpointIdentity,
        root_generation: u64,
        root_sha256: [u8; 32],
        prior_checkpoint_sequence: u64,
        prior_root_sha256: [u8; 32],
        prior_accumulator_digest: [u8; 32],
        prior_cumulative_dropped: u64,
        prior_cumulative_digest: [u8; 32],
        batch_count: u16,
        batch_records_digest: [u8; 32],
        tip: ReleasedDropTipProvenanceV1,
        cumulative_dropped: u64,
        cumulative_digest: [u8; 32],
        terminal: bool,
    ) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        let has_prior = prior_checkpoint_sequence != 0;
        if root_generation == 0
            || root_generation < tip.candidate_root_generation()
            || root_sha256 == [0; 32]
            || cumulative_dropped == 0
            || cumulative_digest == [0; 32]
            || batch_count > 63
            || (batch_count == 0) != (batch_records_digest == [0; 32])
            || has_prior != (prior_root_sha256 != [0; 32])
            || has_prior != (prior_accumulator_digest != [0; 32])
            || has_prior != (prior_cumulative_dropped != 0)
            || has_prior != (prior_cumulative_digest != [0; 32])
            || (has_prior && prior_checkpoint_sequence >= checkpoint.sequence().get())
            || (!has_prior && batch_count == 0)
            || (batch_count == 0
                && (cumulative_dropped != prior_cumulative_dropped
                    || cumulative_digest != prior_cumulative_digest))
            || (batch_count != 0 && cumulative_dropped <= prior_cumulative_dropped)
        {
            return Err(ReleaseCheckpointCertificateDenial::InvalidBinding);
        }
        Ok(Self {
            checkpoint,
            root_generation,
            root_sha256,
            prior_checkpoint_sequence,
            prior_root_sha256,
            prior_accumulator_digest,
            prior_cumulative_dropped,
            prior_cumulative_digest,
            batch_count,
            batch_records_digest,
            tip,
            cumulative_dropped,
            cumulative_digest,
            terminal,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut out = Vec::with_capacity(RELEASE_CHECKPOINT_ACCUMULATOR_WIRE_BYTES);
        write_prefix(&mut out, ACCUMULATOR_KIND);
        self.write_body(&mut out);
        debug_assert_eq!(out.len(), RELEASE_CHECKPOINT_ACCUMULATOR_WIRE_BYTES);
        out
    }

    pub(super) fn write_body(self, out: &mut Vec<u8>) {
        write_checkpoint(out, self.checkpoint);
        out.extend_from_slice(&self.root_generation.to_le_bytes());
        out.extend_from_slice(&self.root_sha256);
        out.extend_from_slice(&self.prior_checkpoint_sequence.to_le_bytes());
        out.extend_from_slice(&self.prior_root_sha256);
        out.extend_from_slice(&self.prior_accumulator_digest);
        out.extend_from_slice(&self.prior_cumulative_dropped.to_le_bytes());
        out.extend_from_slice(&self.prior_cumulative_digest);
        out.extend_from_slice(&self.batch_count.to_le_bytes());
        out.extend_from_slice(&self.batch_records_digest);
        self.tip.write(out);
        out.extend_from_slice(&self.cumulative_dropped.to_le_bytes());
        out.extend_from_slice(&self.cumulative_digest);
        out.push(u8::from(self.terminal));
    }

    pub(super) fn read(
        cursor: &mut Cursor<'_>,
    ) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        let checkpoint = read_checkpoint(cursor)?;
        let root_generation = u64::from_le_bytes(cursor.take()?);
        let root_sha256 = cursor.take()?;
        let prior_checkpoint_sequence = u64::from_le_bytes(cursor.take()?);
        let prior_root_sha256 = cursor.take()?;
        let prior_accumulator_digest = cursor.take()?;
        let prior_cumulative_dropped = u64::from_le_bytes(cursor.take()?);
        let prior_cumulative_digest = cursor.take()?;
        let batch_count = u16::from_le_bytes(cursor.take()?);
        let batch_records_digest = cursor.take()?;
        let tip = ReleasedDropTipProvenanceV1::read(cursor)?;
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
            prior_checkpoint_sequence,
            prior_root_sha256,
            prior_accumulator_digest,
            prior_cumulative_dropped,
            prior_cumulative_digest,
            batch_count,
            batch_records_digest,
            tip,
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
    pub const fn prior_checkpoint_sequence(self) -> u64 {
        self.prior_checkpoint_sequence
    }
    pub const fn prior_root_sha256(self) -> [u8; 32] {
        self.prior_root_sha256
    }
    pub const fn prior_accumulator_digest(self) -> [u8; 32] {
        self.prior_accumulator_digest
    }
    pub const fn prior_cumulative_dropped(self) -> u64 {
        self.prior_cumulative_dropped
    }
    pub const fn prior_cumulative_digest(self) -> [u8; 32] {
        self.prior_cumulative_digest
    }
    pub const fn batch_count(self) -> u16 {
        self.batch_count
    }
    pub const fn batch_records_digest(self) -> [u8; 32] {
        self.batch_records_digest
    }
    pub const fn tip(self) -> ReleasedDropTipProvenanceV1 {
        self.tip
    }
    pub const fn tip_descriptor(self) -> crate::PersistedRecordIdentity {
        self.tip.descriptor_record()
    }
    pub const fn tip_descriptor_frame_sha256(self) -> [u8; 32] {
        self.tip.descriptor_frame_sha256()
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
}
