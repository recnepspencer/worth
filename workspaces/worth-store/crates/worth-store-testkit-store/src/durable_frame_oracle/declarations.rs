//! Independent durable-frame geometry used by integration evidence.

pub use {std::path::Path};

pub const HEADER_BYTES: usize = 48;
pub const CHECKSUM_OFFSET: usize = 44;

/// Byte geometry of the first allocation in an arena, independently decoded
/// from the manifest's wire fields. Ordinal zero denotes its first chunk.
pub fn first_arena_chunk_offset(bytes: &[u8], ordinal: u64) -> u64 {
    assert!(bytes.len() >= 104);
    let alignment = u64::from_le_bytes(bytes[96..104].try_into().unwrap());
    let maximum_frame = u32::from_le_bytes(bytes[88..92].try_into().unwrap()) as u64;
    assert!(alignment.is_power_of_two());
    assert!(maximum_frame > 112);
    104_u64.div_ceil(alignment) * alignment
        + ordinal * maximum_frame.div_ceil(alignment) * alignment
}

pub fn payload(bytes: &[u8]) -> &[u8] {
    &bytes[HEADER_BYTES..]
}

pub fn payload_mut(bytes: &mut [u8]) -> &mut [u8] {
    &mut bytes[HEADER_BYTES..]
}

pub fn reseal(bytes: &mut [u8]) {
    let checksum = independent_crc32c(&[&bytes[..CHECKSUM_OFFSET], &bytes[HEADER_BYTES..]]);
    bytes[CHECKSUM_OFFSET..HEADER_BYTES].copy_from_slice(&checksum.to_le_bytes());
}

pub fn artifact_bytes(root: &Path, relative_paths: &[&str]) -> u64 {
    relative_paths
        .iter()
        .map(|path| std::fs::metadata(root.join(path)).unwrap().len())
        .sum()
}

pub fn independent_crc32c(parts: &[&[u8]]) -> u32 {
    let mut crc = !0_u32;
    for part in parts {
        for byte in *part {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0x82f6_3b78 & 0_u32.wrapping_sub(crc & 1));
            }
        }
    }
    !crc
}