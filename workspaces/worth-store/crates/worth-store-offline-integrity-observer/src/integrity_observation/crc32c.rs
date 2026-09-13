const REVERSED_CASTAGNOLI_POLYNOMIAL: u32 = 0x82f6_3b78;

pub(crate) fn crc32c(parts: &[&[u8]]) -> u32 {
    let mut crc = u32::MAX;
    for byte in parts.iter().flat_map(|part| part.iter()) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (REVERSED_CASTAGNOLI_POLYNOMIAL & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    #[test]
    fn crc32c_matches_the_castagnoli_check_value() {
        assert_eq!(super::crc32c(&[b"123456789"]), 0xe306_9283);
    }
}
