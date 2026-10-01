//! Positive checkpoint custody for a Store that has never admitted a released
//! generation drop. Absence of tag-7 records is not such a proof.

use super::{
    read_checkpoint, write_checkpoint, write_prefix, Cursor, PhysicalCheckpointIdentity,
    ReleaseCheckpointCertificateDenial, NO_RELEASE_KIND, PREFIX_BYTES,
};

pub const RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES: usize =
    PREFIX_BYTES + 24 + 8 + 32 + 8 + 32 + 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseCheckpointNoReleaseV1 {
    checkpoint: PhysicalCheckpointIdentity,
    root_generation: u64,
    root_sha256: [u8; 32],
    prior_checkpoint_sequence: u64,
    prior_root_sha256: [u8; 32],
    prior_marker_payload_sha256: [u8; 32],
}

impl ReleaseCheckpointNoReleaseV1 {
    pub fn new(
        checkpoint: PhysicalCheckpointIdentity,
        root_generation: u64,
        root_sha256: [u8; 32],
        prior_checkpoint_sequence: u64,
        prior_root_sha256: [u8; 32],
        prior_marker_payload_sha256: [u8; 32],
    ) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        let has_prior = prior_checkpoint_sequence != 0;
        if root_generation == 0
            || root_sha256 == [0; 32]
            || has_prior != (prior_root_sha256 != [0; 32])
            || has_prior != (prior_marker_payload_sha256 != [0; 32])
            || (has_prior && prior_checkpoint_sequence >= checkpoint.sequence().get())
        {
            return Err(ReleaseCheckpointCertificateDenial::InvalidBinding);
        }
        Ok(Self {
            checkpoint,
            root_generation,
            root_sha256,
            prior_checkpoint_sequence,
            prior_root_sha256,
            prior_marker_payload_sha256,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        self.encode_in_reserved(Vec::with_capacity(RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES))
            .expect("fixed NoRelease marker wire capacity")
    }

    /// Reuses caller-admitted backing; insufficient capacity performs no write.
    pub fn encode_in_reserved(self, mut out: Vec<u8>) -> Option<Vec<u8>> {
        if out.capacity() < RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES {
            return None;
        }
        out.clear();
        write_prefix(&mut out, NO_RELEASE_KIND);
        write_checkpoint(&mut out, self.checkpoint);
        out.extend_from_slice(&self.root_generation.to_le_bytes());
        out.extend_from_slice(&self.root_sha256);
        out.extend_from_slice(&self.prior_checkpoint_sequence.to_le_bytes());
        out.extend_from_slice(&self.prior_root_sha256);
        out.extend_from_slice(&self.prior_marker_payload_sha256);
        debug_assert_eq!(out.len(), RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES);
        Some(out)
    }

    pub(super) fn read(
        cursor: &mut Cursor<'_>,
    ) -> Result<Self, ReleaseCheckpointCertificateDenial> {
        Self::new(
            read_checkpoint(cursor)?,
            u64::from_le_bytes(cursor.take()?),
            cursor.take()?,
            u64::from_le_bytes(cursor.take()?),
            cursor.take()?,
            cursor.take()?,
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
    pub const fn prior_marker_payload_sha256(self) -> [u8; 32] {
        self.prior_marker_payload_sha256
    }
}
