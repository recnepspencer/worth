//! Fixed little-endian header shared by the two positioned journal families.
use worth_foundational::facade::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolUnsupportedVersion,
    BoundaryProtocolVersion,
};

const VERSION: usize = 8;
const FIRST: usize = VERSION + 2;
const SECOND: usize = FIRST + 8;
pub const DIGEST_OFFSET: usize = SECOND + 8;
const LENGTH: usize = DIGEST_OFFSET + 32;
pub const HEADER_LEN: usize = LENGTH + 8;

pub enum PositionedFrameRefusal {
    TruncatedHeader {
        available: usize,
    },
    UnrecognizedMagic,
    InvalidVersion,
    UnsupportedVersion(BoundaryProtocolUnsupportedVersion),
    SizeLimit {
        declared_payload: u64,
        maximum_frame: usize,
    },
    TruncatedPayload {
        declared: u64,
        available: u64,
    },
    TrailingBytes {
        declared: u64,
        available: u64,
    },
}

pub struct PositionedFrameSpec {
    pub magic: [u8; 8],
    pub version: u16,
    pub maximum_frame: Option<usize>,
}

pub struct PositionedFrameHeader {
    pub first: u64,
    pub second: u64,
    pub digest: [u8; 32],
}

impl PositionedFrameSpec {
    pub fn emit(&self, header: PositionedFrameHeader, payload_len: u64) -> [u8; HEADER_LEN] {
        let mut bytes = [0; HEADER_LEN];
        bytes[..VERSION].copy_from_slice(&self.magic);
        bytes[VERSION..FIRST].copy_from_slice(&self.version.to_le_bytes());
        bytes[FIRST..SECOND].copy_from_slice(&header.first.to_le_bytes());
        bytes[SECOND..DIGEST_OFFSET].copy_from_slice(&header.second.to_le_bytes());
        bytes[DIGEST_OFFSET..LENGTH].copy_from_slice(&header.digest);
        bytes[LENGTH..].copy_from_slice(&payload_len.to_le_bytes());
        bytes
    }

    pub fn validate_version(&self, bytes: &[u8]) -> Result<(), PositionedFrameRefusal> {
        use PositionedFrameRefusal as Refusal;
        let magic = fixed::<8>(bytes, 0)?;
        if magic != self.magic {
            return Err(Refusal::UnrecognizedMagic);
        }
        let version = u16::from_le_bytes(fixed(bytes, VERSION)?);
        let version = BoundaryProtocolVersion::try_new(u32::from(version))
            .map_err(|_| Refusal::InvalidVersion)?;
        let supported = BoundaryProtocolVersion::new(self.version as u32);
        BoundaryProtocolCompatibilityWindow::inclusive(supported, supported)
            .admit(version)
            .map(|_| ())
            .map_err(Refusal::UnsupportedVersion)
    }

    pub fn inspect<'a>(
        &self,
        bytes: &'a [u8],
    ) -> Result<(PositionedFrameHeader, &'a [u8]), PositionedFrameRefusal> {
        use PositionedFrameRefusal as Refusal;
        self.validate_version(bytes)?;
        let header = PositionedFrameHeader {
            first: u64::from_le_bytes(fixed(bytes, FIRST)?),
            second: u64::from_le_bytes(fixed(bytes, SECOND)?),
            digest: fixed(bytes, DIGEST_OFFSET)?,
        };
        let declared = u64::from_le_bytes(fixed(bytes, LENGTH)?);
        if let Some(maximum_frame) = self.maximum_frame
            && declared > maximum_frame.saturating_sub(HEADER_LEN) as u64
        {
            return Err(Refusal::SizeLimit {
                declared_payload: declared,
                maximum_frame,
            });
        }
        let payload = &bytes[HEADER_LEN..];
        let available = payload.len() as u64;
        match available.cmp(&declared) {
            std::cmp::Ordering::Less => Err(Refusal::TruncatedPayload {
                declared,
                available,
            }),
            std::cmp::Ordering::Greater => Err(Refusal::TrailingBytes {
                declared,
                available,
            }),
            std::cmp::Ordering::Equal => Ok((header, payload)),
        }
    }
}

fn fixed<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N], PositionedFrameRefusal> {
    bytes
        .get(offset..offset + N)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(PositionedFrameRefusal::TruncatedHeader {
            available: bytes.len(),
        })
}
