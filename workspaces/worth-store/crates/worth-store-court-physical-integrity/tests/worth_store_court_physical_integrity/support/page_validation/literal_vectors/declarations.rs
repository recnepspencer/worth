pub(crate) use {worth_store_physical_format::PhysicalPageSizeClass};
pub(crate) use {worth_store_physical_integrity::validate_inline_page, worth_store_physical_integrity::InlinePageIntegrityValidation, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::page_validation::independent_crc32c, crate::support::page_validation::page, crate::support::page_validation::page_scope, crate::support::page_validation::store};

pub(crate) fn literal_page(
    page_size: PhysicalPageSizeClass,
    format_bytes: [u8; 10],
    expected_checksum: u32,
) -> Vec<u8> {
    let page_bytes = page_size.bytes() as usize;
    let payload_bytes = page_bytes - 48;
    let mut bytes = vec![0_u8; page_bytes];
    bytes[..8].copy_from_slice(b"WRC5FRM\0");
    bytes[8] = 3;
    bytes[9] = 2;
    bytes[10..20].copy_from_slice(&format_bytes);
    bytes[20..22].copy_from_slice(&48_u16.to_le_bytes());
    bytes[24..28].copy_from_slice(&(payload_bytes as u32).to_le_bytes());
    bytes[28..36].copy_from_slice(&11_u64.to_le_bytes());
    bytes[36..44].copy_from_slice(&0x2122_2324_2526_2728_u64.to_le_bytes());
    bytes[48..56].copy_from_slice(&0x0102_0304_0506_0708_u64.to_le_bytes());
    bytes[56..64].copy_from_slice(&0x1112_1314_1516_1718_u64.to_le_bytes());
    bytes[64..66].copy_from_slice(&1_u16.to_le_bytes());
    bytes[72..88].fill(0xab);
    bytes[88..96].copy_from_slice(&7_u64.to_le_bytes());
    bytes[96..100].copy_from_slice(&((payload_bytes - 3) as u32).to_le_bytes());
    bytes[100..104].copy_from_slice(&3_u32.to_le_bytes());
    bytes[104..112].copy_from_slice(&13_u64.to_le_bytes());
    bytes[page_bytes - 3..].copy_from_slice(&[0xde, 0xad, 0x5a]);
    let checksum = independent_crc32c(&[&bytes[..44], &bytes[48..]]);
    assert_eq!(checksum, expected_checksum, "literal checksum drift");
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
    bytes
}