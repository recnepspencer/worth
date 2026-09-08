const REVERSED_CASTAGNOLI_POLYNOMIAL: u32 = 0x82f6_3b78;

pub(crate) fn refresh_crc32c(bytes: &mut [u8]) {
    let checksum = checksum(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
}

pub(crate) fn checksum(parts: &[&[u8]]) -> u32 {
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

#[test]
fn independent_fixture_crc_matches_the_published_check_value() {
    assert_eq!(checksum(&[b"123456789"]), 0xe306_9283);
}
