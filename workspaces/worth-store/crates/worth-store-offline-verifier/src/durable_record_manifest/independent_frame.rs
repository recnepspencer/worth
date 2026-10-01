use worth_store_physical_format::PhysicalRecordFormatDeclaration;

use super::OfflineDurableManifestDenial;

const FRAME_HEADER_BYTES: usize = 48;
const PAGE_LSN_OFFSET: usize = 36;
const CHECKSUM_OFFSET: usize = 44;
const FRAME_MAGIC: &[u8; 8] = b"WRC5FRM\0";

pub(super) struct IndependentFrame<'bytes> {
    pub(super) identity: u64,
    pub(super) payload: &'bytes [u8],
    pub(super) schema: u8,
}

pub(super) fn decode_frame(
    bytes: &[u8],
    expected_kind: u8,
    expected_format: PhysicalRecordFormatDeclaration,
) -> Result<IndependentFrame<'_>, OfflineDurableManifestDenial> {
    if bytes.len() < FRAME_HEADER_BYTES {
        return Err(OfflineDurableManifestDenial::TruncatedFrame);
    }
    if &bytes[..8] != FRAME_MAGIC
        || bytes[8] != expected_kind
        || !(bytes[9] == 2 || ((expected_kind == 2 || expected_kind == 8) && bytes[9] == 3))
        || bytes[10..20] != expected_format.canonical_identity_bytes()
        || bytes[22..24] != [0; 2]
        || (expected_kind != 3
            && expected_kind != 4
            && bytes[PAGE_LSN_OFFSET..CHECKSUM_OFFSET] != [0; 8])
    {
        return Err(OfflineDurableManifestDenial::FrameDeclarationMismatch);
    }
    let header_bytes = u16::from_le_bytes(bytes[20..22].try_into().unwrap()) as usize;
    let payload_bytes = u32::from_le_bytes(bytes[24..28].try_into().unwrap()) as usize;
    if header_bytes != FRAME_HEADER_BYTES || bytes.len() != header_bytes + payload_bytes {
        return Err(OfflineDurableManifestDenial::FrameLengthMismatch);
    }
    let stored = u32::from_le_bytes(
        bytes[CHECKSUM_OFFSET..FRAME_HEADER_BYTES]
            .try_into()
            .unwrap(),
    );
    let actual = checksum_parts(&[&bytes[..CHECKSUM_OFFSET], &bytes[FRAME_HEADER_BYTES..]]);
    if stored != actual {
        return Err(OfflineDurableManifestDenial::FrameIntegrityMismatch);
    }
    Ok(IndependentFrame {
        identity: u64::from_le_bytes(bytes[28..36].try_into().unwrap()),
        payload: &bytes[FRAME_HEADER_BYTES..],
        schema: bytes[9],
    })
}

#[cfg(test)]
#[path = "independent_frame/tests.rs"]
mod tests;

pub(super) fn artifact_checksum(bytes: &[u8]) -> u32 {
    checksum_parts(&[bytes])
}

fn checksum_parts(parts: &[&[u8]]) -> u32 {
    let mut crc = !0_u32;
    for part in parts {
        for byte in *part {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                let mask = 0_u32.wrapping_sub(crc & 1);
                crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
            }
        }
    }
    !crc
}
