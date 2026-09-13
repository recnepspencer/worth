use super::{WalSegmentIdentity, WAL_FRAME_V1_HEADER_BYTES, WAL_FRAME_V1_VERSION};

const MAGIC: &[u8; 8] = b"WORTHWAL";

/// Descriptive fields decoded from one WAL v1 header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalFrameV1Header {
    identity: WalSegmentIdentity,
    lsn_start: u64,
    lsn_end: u64,
    payload_bytes: u64,
    identity_digest: [u8; 32],
    payload_digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct UnverifiedWalFrameV1Header {
    segment_id: u64,
    generation: u64,
    lsn_start: u64,
    lsn_end: u64,
    payload_bytes: u64,
    identity_digest: [u8; 32],
    payload_digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalFrameV1Denial {
    WrongMagic,
    UnsupportedVersion(u16),
    HeaderLengthMismatch(u16),
    InvalidSegmentIdentity,
    InvalidGeneration,
    InvalidLsnRange,
    EmptyPayload,
    PayloadLengthMismatch,
    ChecksumMismatch,
}

pub fn decode_wal_frame_v1_header(
    header: &[u8; WAL_FRAME_V1_HEADER_BYTES],
) -> Result<WalFrameV1Header, WalFrameV1Denial> {
    decode_unverified_wal_frame_v1_header(header)?.admit()
}

pub(super) fn decode_unverified_wal_frame_v1_header(
    header: &[u8; WAL_FRAME_V1_HEADER_BYTES],
) -> Result<UnverifiedWalFrameV1Header, WalFrameV1Denial> {
    if &header[..8] != MAGIC {
        return Err(WalFrameV1Denial::WrongMagic);
    }
    let version = read_u16(header, 8);
    if version != WAL_FRAME_V1_VERSION {
        return Err(WalFrameV1Denial::UnsupportedVersion(version));
    }
    let header_bytes = read_u16(header, 10);
    if header_bytes as usize != WAL_FRAME_V1_HEADER_BYTES {
        return Err(WalFrameV1Denial::HeaderLengthMismatch(header_bytes));
    }
    Ok(UnverifiedWalFrameV1Header {
        segment_id: read_u64(header, 12),
        generation: read_u64(header, 20),
        lsn_start: read_u64(header, 28),
        lsn_end: read_u64(header, 36),
        payload_bytes: read_u64(header, 44),
        identity_digest: header[52..84].try_into().expect("fixed identity digest"),
        payload_digest: header[84..116].try_into().expect("fixed payload digest"),
    })
}

impl UnverifiedWalFrameV1Header {
    pub(super) fn admit(self) -> Result<WalFrameV1Header, WalFrameV1Denial> {
        if self.segment_id == 0 {
            return Err(WalFrameV1Denial::InvalidSegmentIdentity);
        }
        if self.generation == 0 {
            return Err(WalFrameV1Denial::InvalidGeneration);
        }
        let identity = WalSegmentIdentity::new(self.segment_id, self.generation)
            .expect("nonzero WAL segment coordinates form canonical identity");
        if self.lsn_start >= self.lsn_end {
            return Err(WalFrameV1Denial::InvalidLsnRange);
        }
        if self.payload_bytes == 0 {
            return Err(WalFrameV1Denial::EmptyPayload);
        }
        Ok(WalFrameV1Header {
            identity,
            lsn_start: self.lsn_start,
            lsn_end: self.lsn_end,
            payload_bytes: self.payload_bytes,
            identity_digest: self.identity_digest,
            payload_digest: self.payload_digest,
        })
    }

    pub(super) const fn payload_bytes(self) -> u64 {
        self.payload_bytes
    }

    pub(super) const fn payload_digest(self) -> [u8; 32] {
        self.payload_digest
    }
}

impl WalFrameV1Header {
    pub const fn identity(self) -> WalSegmentIdentity {
        self.identity
    }

    pub const fn segment_id(self) -> u64 {
        self.identity.segment().get()
    }

    pub const fn generation(self) -> u64 {
        self.identity.generation().get()
    }

    pub const fn lsn_start(self) -> u64 {
        self.lsn_start
    }

    pub const fn lsn_end(self) -> u64 {
        self.lsn_end
    }

    pub const fn payload_bytes(self) -> u64 {
        self.payload_bytes
    }

    pub const fn identity_digest(self) -> [u8; 32] {
        self.identity_digest
    }

    pub const fn payload_digest(self) -> [u8; 32] {
        self.payload_digest
    }
}

pub(super) fn write_header(
    segment_id: u64,
    generation: u64,
    lsn_start: u64,
    lsn_end: u64,
    identity_digest: [u8; 32],
    payload_digest: [u8; 32],
    payload_bytes: usize,
) -> [u8; WAL_FRAME_V1_HEADER_BYTES] {
    let mut header = [0; WAL_FRAME_V1_HEADER_BYTES];
    header[..8].copy_from_slice(MAGIC);
    header[8..10].copy_from_slice(&WAL_FRAME_V1_VERSION.to_le_bytes());
    header[10..12].copy_from_slice(&(WAL_FRAME_V1_HEADER_BYTES as u16).to_le_bytes());
    header[12..20].copy_from_slice(&segment_id.to_le_bytes());
    header[20..28].copy_from_slice(&generation.to_le_bytes());
    header[28..36].copy_from_slice(&lsn_start.to_le_bytes());
    header[36..44].copy_from_slice(&lsn_end.to_le_bytes());
    header[44..52].copy_from_slice(&(payload_bytes as u64).to_le_bytes());
    header[52..84].copy_from_slice(&identity_digest);
    header[84..116].copy_from_slice(&payload_digest);
    header
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("fixed field"))
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("fixed field"))
}
