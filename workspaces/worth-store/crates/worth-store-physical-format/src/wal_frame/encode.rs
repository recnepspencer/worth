use sha2::{Digest, Sha256};

use super::header::write_header;
use super::{
    wal_frame_v1_declared_identity_digest, WalFrameV1Denial, WalSegmentIdentity,
    WAL_FRAME_V1_FOOTER_BYTES, WAL_FRAME_V1_HEADER_BYTES,
};

/// Descriptive request for canonical WAL v1 byte construction.
pub struct WalFrameV1EncodeRequest<'a> {
    identity: WalSegmentIdentity,
    lsn_start: u64,
    lsn_end: u64,
    declared_identity: &'a [u8],
    payload: &'a [u8],
}

impl<'a> WalFrameV1EncodeRequest<'a> {
    pub const fn new(
        segment_id: u64,
        generation: u64,
        lsn_start: u64,
        lsn_end: u64,
        declared_identity: &'a [u8],
        payload: &'a [u8],
    ) -> Result<Self, WalFrameV1Denial> {
        if segment_id == 0 {
            return Err(WalFrameV1Denial::InvalidSegmentIdentity);
        }
        if generation == 0 {
            return Err(WalFrameV1Denial::InvalidGeneration);
        }
        let identity = WalSegmentIdentity::new(segment_id, generation)
            .expect("nonzero WAL segment coordinates form canonical identity");
        if lsn_start >= lsn_end {
            return Err(WalFrameV1Denial::InvalidLsnRange);
        }
        if payload.is_empty() {
            return Err(WalFrameV1Denial::EmptyPayload);
        }
        Ok(Self {
            identity,
            lsn_start,
            lsn_end,
            declared_identity,
            payload,
        })
    }

    pub const fn from_segment_identity(
        identity: WalSegmentIdentity,
        lsn_start: u64,
        lsn_end: u64,
        declared_identity: &'a [u8],
        payload: &'a [u8],
    ) -> Result<Self, WalFrameV1Denial> {
        Self::new(
            identity.segment().get(),
            identity.generation().get(),
            lsn_start,
            lsn_end,
            declared_identity,
            payload,
        )
    }
}

pub fn encode_wal_frame_v1(request: WalFrameV1EncodeRequest<'_>) -> Vec<u8> {
    let identity_digest = wal_frame_v1_declared_identity_digest(request.declared_identity);
    let payload_digest = Sha256::digest(request.payload).into();
    let header = write_header(
        request.identity.segment().get(),
        request.identity.generation().get(),
        request.lsn_start,
        request.lsn_end,
        identity_digest,
        payload_digest,
        request.payload.len(),
    );
    let mut frame = Vec::with_capacity(
        WAL_FRAME_V1_HEADER_BYTES + request.payload.len() + WAL_FRAME_V1_FOOTER_BYTES,
    );
    frame.extend_from_slice(&header);
    frame.extend_from_slice(request.payload);
    let footer = Sha256::digest(&frame);
    frame.extend_from_slice(&footer);
    frame
}
