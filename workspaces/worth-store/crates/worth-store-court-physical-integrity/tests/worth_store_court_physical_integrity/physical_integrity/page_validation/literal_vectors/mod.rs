#[allow(unused_imports)] use crate::page_validation::literal_vectors::*;
use {worth_store_physical_format::PhysicalPageSizeClass};
use {worth_store_physical_integrity::validate_inline_page, worth_store_physical_integrity::InlinePageIntegrityValidation, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::support::page_validation::independent_crc32c, crate::support::page_validation::page, crate::support::page_validation::page_scope, crate::support::page_validation::store};

#[test]
fn independent_literal_page_vectors_cover_each_declared_size() {
    let vectors = [
        (
            worth_store_physical_format::PhysicalPageSizeClass::KiB16,
            [2, 0, 0, 64, 0, 0, 1, 1, 1, 24],
            0x6363_1ebf_u32,
        ),
        (
            worth_store_physical_format::PhysicalPageSizeClass::KiB32,
            [2, 0, 0, 128, 0, 0, 1, 1, 1, 24],
            0x8018_e3f4_u32,
        ),
        (
            worth_store_physical_format::PhysicalPageSizeClass::KiB64,
            [2, 0, 0, 0, 1, 0, 1, 1, 1, 24],
            0x290f_03a0_u32,
        ),
    ];
    for (page_size, format_bytes, expected_checksum) in vectors {
        let bytes = literal_page(page_size, format_bytes, expected_checksum);
        let identity = page(0x0102_0304_0506_0708, 0x1112_1314_1516_1718, 11);
        let scope = page_scope(store(0x44), page_size, identity);
        let (validation, _) =
            validate_inline_page(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes), scope);
        let worth_store_physical_integrity::InlinePageIntegrityValidation::Intact(validated) = validation else {
            panic!("independent literal page rejected at {page_size:?}");
        };
        assert_eq!(validated.page_identity(), identity);
        assert_eq!(validated.slot_count(), 1);
        assert_eq!(validated.free_bytes(), page_size.bytes() - 48 - 24 - 40 - 3);
    }
}
