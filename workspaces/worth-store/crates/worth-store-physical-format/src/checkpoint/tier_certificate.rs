//! Selected-checkpoint ratchet for one durable tier-epoch activation.
//! The record identifies C9-admitted WAL frames; C8 must join those frames
//! before accepting a fold and must bind this record to the selected checkpoint.

use sha2::{Digest, Sha256};

use super::identity::{decode_identity, encode_identity};
use super::PhysicalCheckpointIdentity;
use crate::{TierEpochActivationPhaseV1, TierEpochActivationV1, TIER_EPOCH_ACTIVATION_WIRE_BYTES};

const DOMAIN: &[u8] = b"store.physical.checkpoint.tier-epoch-custody.v1";
const WAL_WITNESS_BYTES: usize = 80;
const WIRE_BYTES: usize =
    8 + DOMAIN.len() + 24 + 8 + 32 + 32 + TIER_EPOCH_ACTIVATION_WIRE_BYTES + 2 * WAL_WITNESS_BYTES;
pub const TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES: usize = WIRE_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TierEpochCheckpointCertificateDenial {
    Malformed,
    InvalidBinding,
}

/// Exact selected C9 frame identity and payload digests for one typed WAL member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierEpochWalFrameWitnessV1 {
    lsn_start: u64,
    lsn_end_exclusive: u64,
    identity_digest: [u8; 32],
    payload_digest: [u8; 32],
}

impl TierEpochWalFrameWitnessV1 {
    pub fn new(
        lsn_start: u64,
        lsn_end_exclusive: u64,
        identity_digest: [u8; 32],
        payload_digest: [u8; 32],
    ) -> Option<Self> {
        (lsn_start > 0
            && lsn_start < lsn_end_exclusive
            && identity_digest != [0; 32]
            && payload_digest != [0; 32])
            .then_some(Self {
                lsn_start,
                lsn_end_exclusive,
                identity_digest,
                payload_digest,
            })
    }

    pub const fn lsn_start(self) -> u64 {
        self.lsn_start
    }
    pub const fn lsn_end_exclusive(self) -> u64 {
        self.lsn_end_exclusive
    }
    pub const fn identity_digest(self) -> [u8; 32] {
        self.identity_digest
    }
    pub const fn payload_digest(self) -> [u8; 32] {
        self.payload_digest
    }

    fn encode_into(self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.lsn_start.to_le_bytes());
        bytes.extend_from_slice(&self.lsn_end_exclusive.to_le_bytes());
        bytes.extend_from_slice(&self.identity_digest);
        bytes.extend_from_slice(&self.payload_digest);
    }

    fn decode(bytes: &[u8], cursor: &mut usize) -> Option<Self> {
        Self::new(
            u64::from_le_bytes(take::<8>(bytes, cursor)),
            u64::from_le_bytes(take::<8>(bytes, cursor)),
            take::<32>(bytes, cursor),
            take::<32>(bytes, cursor),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierEpochCheckpointCertificateV1 {
    checkpoint: PhysicalCheckpointIdentity,
    root_generation: u64,
    root_sha256: [u8; 32],
    anchor: [u8; 32],
    intent: TierEpochActivationV1,
    intent_frame: TierEpochWalFrameWitnessV1,
    completed_frame: TierEpochWalFrameWitnessV1,
}

impl TierEpochCheckpointCertificateV1 {
    pub fn new(
        checkpoint: PhysicalCheckpointIdentity,
        root_generation: u64,
        root_sha256: [u8; 32],
        anchor: [u8; 32],
        intent: TierEpochActivationV1,
        intent_frame: TierEpochWalFrameWitnessV1,
        completed_frame: TierEpochWalFrameWitnessV1,
    ) -> Result<Self, TierEpochCheckpointCertificateDenial> {
        let intent_payload_digest: [u8; 32] = Sha256::digest(intent.encode()).into();
        let completed_payload_digest: [u8; 32] = Sha256::digest(intent.completed().encode()).into();
        if checkpoint.store_identity().bytes() != intent.store()
            || root_generation < intent.candidate_root_generation()
            || root_sha256 == [0; 32]
            || anchor != intent.epoch_anchor()
            || intent.phase() != TierEpochActivationPhaseV1::Intent
            || intent_frame.lsn_end_exclusive() > completed_frame.lsn_start()
            || intent_frame.payload_digest() != intent_payload_digest
            || completed_frame.payload_digest() != completed_payload_digest
        {
            return Err(TierEpochCheckpointCertificateDenial::InvalidBinding);
        }
        if root_generation == intent.candidate_root_generation()
            && root_sha256 != intent.candidate_root_sha256()
        {
            return Err(TierEpochCheckpointCertificateDenial::InvalidBinding);
        }
        Ok(Self {
            checkpoint,
            root_generation,
            root_sha256,
            anchor,
            intent,
            intent_frame,
            completed_frame,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(WIRE_BYTES);
        bytes.extend_from_slice(&(DOMAIN.len() as u64).to_le_bytes());
        bytes.extend_from_slice(DOMAIN);
        let mut identity = [0; 24];
        encode_identity(&mut identity, self.checkpoint);
        bytes.extend_from_slice(&identity);
        bytes.extend_from_slice(&self.root_generation.to_le_bytes());
        bytes.extend_from_slice(&self.root_sha256);
        bytes.extend_from_slice(&self.anchor);
        bytes.extend_from_slice(&self.intent.encode());
        self.intent_frame.encode_into(&mut bytes);
        self.completed_frame.encode_into(&mut bytes);
        debug_assert_eq!(bytes.len(), WIRE_BYTES);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, TierEpochCheckpointCertificateDenial> {
        if bytes.len() != WIRE_BYTES
            || bytes[..8] != (DOMAIN.len() as u64).to_le_bytes()
            || &bytes[8..8 + DOMAIN.len()] != DOMAIN
        {
            return Err(TierEpochCheckpointCertificateDenial::Malformed);
        }
        let mut cursor = 8 + DOMAIN.len();
        let checkpoint = decode_identity(&bytes[cursor..cursor + 24])
            .map_err(|_| TierEpochCheckpointCertificateDenial::Malformed)?;
        cursor += 24;
        let root_generation = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let root_sha256 = take::<32>(bytes, &mut cursor);
        let anchor = take::<32>(bytes, &mut cursor);
        let intent = TierEpochActivationV1::decode(
            &bytes[cursor..cursor + TIER_EPOCH_ACTIVATION_WIRE_BYTES],
        )
        .map_err(|_| TierEpochCheckpointCertificateDenial::Malformed)?;
        cursor += TIER_EPOCH_ACTIVATION_WIRE_BYTES;
        let intent_frame = TierEpochWalFrameWitnessV1::decode(bytes, &mut cursor)
            .ok_or(TierEpochCheckpointCertificateDenial::Malformed)?;
        let completed_frame = TierEpochWalFrameWitnessV1::decode(bytes, &mut cursor)
            .ok_or(TierEpochCheckpointCertificateDenial::Malformed)?;
        debug_assert_eq!(cursor, bytes.len());
        Self::new(
            checkpoint,
            root_generation,
            root_sha256,
            anchor,
            intent,
            intent_frame,
            completed_frame,
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
    pub const fn anchor(self) -> [u8; 32] {
        self.anchor
    }
    pub const fn intent(self) -> TierEpochActivationV1 {
        self.intent
    }
    pub const fn intent_frame(self) -> TierEpochWalFrameWitnessV1 {
        self.intent_frame
    }
    pub const fn completed_frame(self) -> TierEpochWalFrameWitnessV1 {
        self.completed_frame
    }
}

fn take<const N: usize>(bytes: &[u8], cursor: &mut usize) -> [u8; N] {
    let result = bytes[*cursor..*cursor + N].try_into().unwrap();
    *cursor += N;
    result
}
