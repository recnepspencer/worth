use super::header::decode_unverified_wal_frame_v1_header;
use super::{
    WalFrameV1ChecksumCalculator, WalFrameV1Denial, WalFrameV1Header, WAL_FRAME_V1_FOOTER_BYTES,
    WAL_FRAME_V1_HEADER_BYTES,
};

/// One exact, checksum-admitted WAL v1 frame borrowed from bounded bytes.
///
/// Segment identity and LSN fields become observable only after both declared
/// checksum relations and the complete framing relation have passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedWalFrameV1<'frame> {
    header: WalFrameV1Header,
    payload: &'frame [u8],
    frame_digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalFrameV1BoundedDecodeDenial {
    TruncatedHeader,
    Header(WalFrameV1Denial),
    FrameLengthOverflow,
    FrameLengthMismatch { declared: u64, observed: u64 },
    ChecksumMismatch(WalFrameV1ChecksumMismatch),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalFrameV1ChecksumMismatch {
    payload_checksum: bool,
    frame_checksum: bool,
}

pub fn decode_bounded_wal_frame_v1(
    bytes: &[u8],
) -> Result<BoundedWalFrameV1<'_>, WalFrameV1BoundedDecodeDenial> {
    let header_bytes: &[u8; WAL_FRAME_V1_HEADER_BYTES] = bytes
        .get(..WAL_FRAME_V1_HEADER_BYTES)
        .and_then(|header| header.try_into().ok())
        .ok_or(WalFrameV1BoundedDecodeDenial::TruncatedHeader)?;
    let unverified = decode_unverified_wal_frame_v1_header(header_bytes)
        .map_err(WalFrameV1BoundedDecodeDenial::Header)?;
    let declared_frame_bytes = (WAL_FRAME_V1_HEADER_BYTES as u64)
        .checked_add(unverified.payload_bytes())
        .and_then(|length| length.checked_add(WAL_FRAME_V1_FOOTER_BYTES as u64))
        .ok_or(WalFrameV1BoundedDecodeDenial::FrameLengthOverflow)?;
    let observed_frame_bytes = bytes.len() as u64;
    if declared_frame_bytes != observed_frame_bytes {
        return Err(WalFrameV1BoundedDecodeDenial::FrameLengthMismatch {
            declared: declared_frame_bytes,
            observed: observed_frame_bytes,
        });
    }

    let payload_end = WAL_FRAME_V1_HEADER_BYTES
        + usize::try_from(unverified.payload_bytes())
            .map_err(|_| WalFrameV1BoundedDecodeDenial::FrameLengthOverflow)?;
    let payload = &bytes[WAL_FRAME_V1_HEADER_BYTES..payload_end];
    let footer: &[u8; WAL_FRAME_V1_FOOTER_BYTES] = bytes[payload_end..]
        .try_into()
        .expect("exact bounded WAL framing fixes the footer width");
    let mut calculator = WalFrameV1ChecksumCalculator::new(header_bytes);
    calculator
        .update_payload(payload)
        .map_err(WalFrameV1BoundedDecodeDenial::Header)?;
    let calculated = calculator
        .finish_for_payload_bytes(unverified.payload_bytes())
        .map_err(WalFrameV1BoundedDecodeDenial::Header)?;
    let mismatch = WalFrameV1ChecksumMismatch {
        payload_checksum: calculated.payload() != unverified.payload_digest(),
        frame_checksum: calculated.frame() != *footer,
    };
    if mismatch.any() {
        return Err(WalFrameV1BoundedDecodeDenial::ChecksumMismatch(mismatch));
    }

    let header = unverified
        .admit()
        .map_err(WalFrameV1BoundedDecodeDenial::Header)?;
    Ok(BoundedWalFrameV1 {
        header,
        payload,
        frame_digest: calculated.frame(),
    })
}

impl<'frame> BoundedWalFrameV1<'frame> {
    pub const fn header(self) -> WalFrameV1Header {
        self.header
    }

    pub const fn payload(self) -> &'frame [u8] {
        self.payload
    }

    pub const fn frame_digest(self) -> [u8; 32] {
        self.frame_digest
    }
}

impl WalFrameV1ChecksumMismatch {
    pub const fn payload_checksum(self) -> bool {
        self.payload_checksum
    }

    pub const fn frame_checksum(self) -> bool {
        self.frame_checksum
    }

    const fn any(self) -> bool {
        self.payload_checksum || self.frame_checksum
    }
}
