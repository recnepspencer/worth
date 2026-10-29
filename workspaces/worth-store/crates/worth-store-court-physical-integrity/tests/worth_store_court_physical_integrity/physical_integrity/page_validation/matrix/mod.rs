#[allow(unused_imports)] use crate::page_validation::matrix::*;
use {worth_store_physical_format::encode_inline_page, worth_store_physical_format::PhysicalPageSizeClass};
use {worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PhysicalIntegrityVersionAxis};

use {crate::support::page_validation::assert_damage, crate::support::page_validation::clean_page, crate::support::page_validation::field_range, crate::support::page_validation::format, crate::support::page_validation::page, crate::support::page_validation::page_scope, crate::support::page_validation::reseal, crate::support::page_validation::store, crate::support::page_validation::validate_rejection, crate::support::page_validation::PAGE_OFFSET, crate::support::page_validation::PAGE_SIZES};

#[test]
fn page_b_k_l_s_t_u_matrix_is_exact_at_16_32_and_64_kib() {
    for page_size in PAGE_SIZES {
        let identity = page(3, 5, 7);
        let scope = page_scope(store(9), page_size, identity);

        let mut covered_byte_flip = clean_page(page_size, identity);
        covered_byte_flip[48] ^= 0x40;
        assert_damage(
            &covered_byte_flip,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        let mut checksum_flip = clean_page(page_size, identity);
        checksum_flip[44] ^= 0x01;
        assert_damage(
            &checksum_flip,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        let mut length_lie = clean_page(page_size, identity);
        let payload_length = page_size.bytes() - 49;
        length_lie[24..28].copy_from_slice(&payload_length.to_le_bytes());
        reseal(&mut length_lie);
        assert_damage(
            &length_lie,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
            field_range(scope, 20, 8),
            Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        let generation_substitution = clean_page(page_size, page(3, 5, 8));
        assert_damage(
            &generation_substitution,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            field_range(scope, 28, 8),
            Some(worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );

        let mut zero_generation = clean_page(page_size, identity);
        zero_generation[28..36].copy_from_slice(&0_u64.to_le_bytes());
        reseal(&mut zero_generation);
        assert_damage(
            &zero_generation,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            field_range(scope, 28, 8),
            Some(worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration),
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        let complete = clean_page(page_size, identity);
        let truncated = &complete[..complete.len() - 7];
        assert_damage(
            truncated,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::Truncated,
            worth_store_physical_integrity::PhysicalByteRange::new(PAGE_OFFSET + u64::from(page_size.bytes()) - 7, 7).unwrap(),
            None,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        let mut unsupported_schema = clean_page(page_size, identity);
        unsupported_schema[9] = 3;
        reseal(&mut unsupported_schema);
        assert_unsupported(
            &unsupported_schema,
            scope,
            worth_store_physical_integrity::PhysicalIntegrityVersionAxis::EnvelopeSchema,
            3,
        );

        let mut unsupported_record_version = clean_page(page_size, identity);
        unsupported_record_version[10..12].copy_from_slice(&3_u16.to_le_bytes());
        reseal(&mut unsupported_record_version);
        assert_unsupported(
            &unsupported_record_version,
            scope,
            worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalFormat,
            3,
        );
    }
}

#[test]
fn kind_format_segment_page_and_generation_substitutions_name_the_exact_field() {
    let page_size = worth_store_physical_format::PhysicalPageSizeClass::KiB16;
    let identity = page(3, 5, 7);
    let scope = page_scope(store(9), page_size, identity);

    let mut kind = clean_page(page_size, identity);
    kind[8] = 4;
    reseal(&mut kind);
    assert_damage(
        &kind,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
        field_range(scope, 8, 1),
        Some(worth_store_physical_integrity::PhysicalFormatField::ArtifactFamily),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut other_format = clean_page(page_size, identity);
    other_format[12..16].copy_from_slice(&worth_store_physical_format::PhysicalPageSizeClass::KiB32.bytes().to_le_bytes());
    reseal(&mut other_format);
    assert_damage(
        &other_format,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FormatMismatch,
        field_range(scope, 10, 10),
        Some(worth_store_physical_integrity::PhysicalFormatField::FormatDeclaration),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    for (other, range, field) in [
        (
            page(4, 5, 7),
            field_range(scope, 48, 8),
            worth_store_physical_integrity::PhysicalFormatField::SegmentIdentity,
        ),
        (
            page(3, 6, 7),
            field_range(scope, 56, 8),
            worth_store_physical_integrity::PhysicalFormatField::PageIdentity,
        ),
    ] {
        let substitution = clean_page(page_size, other);
        assert_damage(
            &substitution,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            range,
            Some(field),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );
    }

    for (encoded_range, damage_range, field) in [
        (
            48..56,
            field_range(scope, 48, 8),
            worth_store_physical_integrity::PhysicalFormatField::SegmentIdentity,
        ),
        (
            56..64,
            field_range(scope, 56, 8),
            worth_store_physical_integrity::PhysicalFormatField::PageIdentity,
        ),
    ] {
        let mut zero_identity = clean_page(page_size, identity);
        zero_identity[encoded_range].copy_from_slice(&0_u64.to_le_bytes());
        reseal(&mut zero_identity);
        assert_damage(
            &zero_identity,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            damage_range,
            Some(field),
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );
    }

    let wrong_size_scope = worth_store_physical_integrity::PhysicalArtifactScope::inline_page(
        store(9),
        format(page_size),
        identity,
        worth_store_physical_integrity::PhysicalByteRange::new(PAGE_OFFSET, 256).unwrap(),
    );
    let mut short_but_self_framed =
        encode_inline_page(format(page_size), identity, &[]).unwrap()[..256].to_vec();
    short_but_self_framed[24..28].copy_from_slice(&208_u32.to_le_bytes());
    reseal(&mut short_but_self_framed);
    assert_damage(
        &short_but_self_framed,
        wrong_size_scope,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        field_range(wrong_size_scope, 24, 4),
        Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );
}
