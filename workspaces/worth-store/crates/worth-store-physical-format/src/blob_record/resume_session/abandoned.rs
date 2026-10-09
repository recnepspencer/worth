use std::num::NonZeroU64;

use crate::PersistedRecordIdentity;

use super::super::envelope::{encode, nonzero_16, nonzero_32};
use super::super::{BlobRecordDenial, BlobRecordKind};

const EXPLICIT_PAYLOAD_BYTES: usize = 89;
const EXPIRED_PAYLOAD_BYTES: usize = 97;

/// The selected terminal reason. Expiry names a completed checkpoint witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobAbandonmentReasonV1 {
    ExplicitAbort,
    CheckpointExpired { checkpoint_sequence: NonZeroU64 },
}

/// A Store-published terminal fact. This representation cannot itself prove
/// C.5 selection, absence of publication, or the claimant's authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobSessionAbandonedV1 {
    store: [u8; 16],
    session: [u8; 16],
    declaration_record: PersistedRecordIdentity,
    declaration_digest: [u8; 32],
    reason: BlobAbandonmentReasonV1,
}

impl BlobSessionAbandonedV1 {
    pub fn new(
        store: [u8; 16],
        session: [u8; 16],
        declaration_record: PersistedRecordIdentity,
        declaration_digest: [u8; 32],
        reason: BlobAbandonmentReasonV1,
    ) -> Result<Self, BlobRecordDenial> {
        Ok(Self {
            store: nonzero_16(store)?,
            session: nonzero_16(session)?,
            declaration_record,
            declaration_digest: nonzero_32(declaration_digest)?,
            reason,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(match self.reason {
            BlobAbandonmentReasonV1::ExplicitAbort => EXPLICIT_PAYLOAD_BYTES,
            BlobAbandonmentReasonV1::CheckpointExpired { .. } => EXPIRED_PAYLOAD_BYTES,
        });
        payload.extend_from_slice(&self.store);
        payload.extend_from_slice(&self.session);
        payload.extend_from_slice(&self.declaration_record.allocation_epoch());
        payload.extend_from_slice(&self.declaration_record.ordinal().to_le_bytes());
        payload.extend_from_slice(&self.declaration_digest);
        match self.reason {
            BlobAbandonmentReasonV1::ExplicitAbort => payload.push(1),
            BlobAbandonmentReasonV1::CheckpointExpired {
                checkpoint_sequence,
            } => {
                payload.push(2);
                payload.extend_from_slice(&checkpoint_sequence.get().to_le_bytes());
            }
        }
        encode(BlobRecordKind::SessionAbandoned, &payload)
            .expect("fixed abandonment control fits frame ceiling")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::SessionAbandoned {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload)
    }

    pub(in crate::blob_record) fn decode_payload(payload: &[u8]) -> Result<Self, BlobRecordDenial> {
        if payload.len() < EXPLICIT_PAYLOAD_BYTES {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        let reason = match payload[88] {
            1 if payload.len() == EXPLICIT_PAYLOAD_BYTES => BlobAbandonmentReasonV1::ExplicitAbort,
            2 if payload.len() == EXPIRED_PAYLOAD_BYTES => {
                let sequence =
                    u64::from_le_bytes(payload[89..97].try_into().expect("fixed sequence"));
                BlobAbandonmentReasonV1::CheckpointExpired {
                    checkpoint_sequence: NonZeroU64::new(sequence)
                        .ok_or(BlobRecordDenial::InvalidAbandonment)?,
                }
            }
            1 | 2 => return Err(BlobRecordDenial::LengthMismatch),
            _ => return Err(BlobRecordDenial::InvalidAbandonment),
        };
        let declaration_record = PersistedRecordIdentity::new(
            payload[32..48].try_into().expect("fixed record"),
            u64::from_le_bytes(payload[48..56].try_into().expect("fixed record")),
        )
        .ok_or(BlobRecordDenial::InvalidAbandonment)?;
        Self::new(
            payload[0..16].try_into().expect("fixed field"),
            payload[16..32].try_into().expect("fixed field"),
            declaration_record,
            payload[56..88].try_into().expect("fixed field"),
            reason,
        )
    }

    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn session(self) -> [u8; 16] {
        self.session
    }
    pub const fn declaration_record(self) -> PersistedRecordIdentity {
        self.declaration_record
    }
    pub const fn declaration_digest(self) -> [u8; 32] {
        self.declaration_digest
    }
    pub const fn reason(self) -> BlobAbandonmentReasonV1 {
        self.reason
    }
}
