//! Versioned selected-head roster commitment for a released checkpoint.
//! The roster digest is over the checkpoint-source tree, not the later WAL
//! effective tree. Decoding validates shape; C8 and Store validate membership.

use super::{
    write_prefix_v2, Cursor, ReleaseCheckpointAccumulatorV1, ReleaseCheckpointCertificateDenial,
    ACCUMULATOR_KIND, RELEASE_CHECKPOINT_ACCUMULATOR_WIRE_BYTES,
};

pub const RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES: usize =
    RELEASE_CHECKPOINT_ACCUMULATOR_WIRE_BYTES + 8 + 32 + 8 + 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseCheckpointAccumulatorV2 {
    base: ReleaseCheckpointAccumulatorV1,
    head_count: u64,
    head_roster_digest: [u8; 32],
    prior_head_count: u64,
    prior_head_roster_digest: [u8; 32],
}

impl ReleaseCheckpointAccumulatorV2 {
    pub fn new(
        base: ReleaseCheckpointAccumulatorV1,
        head_count: u64,
        head_roster_digest: [u8; 32],
        prior_head_count: u64,
        prior_head_roster_digest: [u8; 32],
    ) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        if head_roster_digest == [0; 32]
            || (base.prior_checkpoint_sequence() == 0
                && (prior_head_count != 0 || prior_head_roster_digest != [0; 32]))
            || (base.prior_checkpoint_sequence() != 0 && prior_head_roster_digest == [0; 32])
        {
            return Err(ReleaseCheckpointCertificateDenial::InvalidBinding);
        }
        Ok(Self {
            base,
            head_count,
            head_roster_digest,
            prior_head_count,
            prior_head_roster_digest,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        self.encode_in_reserved(Vec::with_capacity(
            RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES,
        ))
        .expect("fixed V2 accumulator wire capacity")
    }

    /// Reuses caller-admitted backing; insufficient capacity performs no write.
    pub fn encode_in_reserved(self, mut out: Vec<u8>) -> Option<Vec<u8>> {
        if out.capacity() < RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES {
            return None;
        }
        out.clear();
        write_prefix_v2(&mut out, ACCUMULATOR_KIND);
        self.base.write_body(&mut out);
        out.extend_from_slice(&self.head_count.to_le_bytes());
        out.extend_from_slice(&self.head_roster_digest);
        out.extend_from_slice(&self.prior_head_count.to_le_bytes());
        out.extend_from_slice(&self.prior_head_roster_digest);
        debug_assert_eq!(out.len(), RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES);
        Some(out)
    }

    pub(super) fn read(
        cursor: &mut Cursor<'_>,
    ) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        let base = ReleaseCheckpointAccumulatorV1::read(cursor)?;
        Self::new(
            base,
            u64::from_le_bytes(cursor.take()?),
            cursor.take()?,
            u64::from_le_bytes(cursor.take()?),
            cursor.take()?,
        )
    }

    pub const fn base(self) -> ReleaseCheckpointAccumulatorV1 {
        self.base
    }

    pub const fn head_count(self) -> u64 {
        self.head_count
    }

    pub const fn head_roster_digest(self) -> [u8; 32] {
        self.head_roster_digest
    }

    pub const fn prior_head_count(self) -> u64 {
        self.prior_head_count
    }

    pub const fn prior_head_roster_digest(self) -> [u8; 32] {
        self.prior_head_roster_digest
    }
}
