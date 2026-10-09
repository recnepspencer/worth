//! Independent bounded scalar interpretation shared by blob fact families.

pub(super) fn admitted_chunk_size(value: u32) -> bool {
    ((64_u32 << 10)..=(256_u32 << 10)).contains(&value)
}

pub(super) fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(super) fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

pub(super) fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
