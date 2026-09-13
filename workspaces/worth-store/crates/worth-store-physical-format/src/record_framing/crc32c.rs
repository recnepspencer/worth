#[cfg(feature = "certification-test-authority")]
std::thread_local! {
    static CHECKSUM_INVOCATIONS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Actual CRC32C calls on this thread; certification evidence, never admission authority.
#[cfg(feature = "certification-test-authority")]
pub fn certification_crc32c_invocations() -> u64 {
    CHECKSUM_INVOCATIONS.with(std::cell::Cell::get)
}

pub(crate) fn checksum(parts: &[&[u8]]) -> u32 {
    #[cfg(feature = "certification-test-authority")]
    CHECKSUM_INVOCATIONS.with(|count| count.set(count.get().saturating_add(1)));
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
