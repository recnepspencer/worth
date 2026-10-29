#[allow(unused_imports)]
use crate::checkpoint_validation::record_matrix::*;

#[test]
fn four_pre_footer_kinds_enforce_b_k_l_t_u_and_kind_with_exact_localization() {
    for kind in crate::checkpoint_validation::record_matrix::Kind::ALL {
        let scope = kind.scope();
        let mut covered_flip = kind.bytes();
        covered_flip[16] ^= 1;
        assert_record_damage(
            kind,
            &covered_flip,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        let mut checksum_flip = kind.bytes();
        let last = checksum_flip.len() - 1;
        checksum_flip[last] ^= 1;
        assert_record_damage(
            kind,
            &checksum_flip,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        let mut length_lie = kind.bytes();
        length_lie[12..16].copy_from_slice(&1_u32.to_le_bytes());
        reseal_record(&mut length_lie);
        assert_record_damage(
            kind,
            &length_lie,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
            field_range(scope, 12, 4),
            Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        let mut kind_substitution = kind.bytes();
        kind_substitution[9] = if kind_substitution[9] == 5 { 4 } else { 5 };
        reseal_record(&mut kind_substitution);
        assert_record_damage(
            kind,
            &kind_substitution,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::RecordKindMismatch,
            field_range(scope, 9, 1),
            Some(worth_store_physical_integrity::PhysicalFormatField::CheckpointRecordKind),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );

        let complete = kind.bytes();
        let truncated = &complete[..complete.len() - 1];
        assert_record_damage(
            kind,
            truncated,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::Truncated,
            worth_store_physical_integrity::PhysicalByteRange::new(
                scope.byte_range().end_exclusive() - 1,
                1,
            )
            .unwrap(),
            None,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        // Schemas 2 and 3 are admitted; 4 remains outside the version window.
        let mut unsupported = kind.bytes();
        unsupported[8] = 4;
        reseal_record(&mut unsupported);
        let (rejection, counters) = rejection(kind, &unsupported, scope);
        let worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(version) =
            rejection
        else {
            panic!("unsupported checkpoint schema collapsed into damage");
        };
        assert_eq!(version.scope(), scope);
        assert_eq!(
            version.axis(),
            worth_store_physical_integrity::PhysicalIntegrityVersionAxis::CheckpointRecordSchema
        );
        assert_eq!(version.observed(), 4);
        assert_rejected_counters(counters, kind.family(), unsupported.len() as u64, None);
    }
}

#[test]
fn header_stages_only_sequence_and_known_identity_rejects_substitution() {
    let staged = validate_header(&HEADER, header_scope_staged());
    assert_eq!(staged.checkpoint_identity(), identity());

    let other_sequence = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        store_for_identity(),
        std::num::NonZeroU64::new(8).unwrap(),
    );
    let scope = header_scope_known(other_sequence);
    let (sequence_rejection, _) = rejection(
        crate::checkpoint_validation::record_matrix::Kind::Header,
        &HEADER,
        scope,
    );
    assert_damage(
        sequence_rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::SequenceMismatch,
        field_range(scope, 32, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::CheckpointIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let other_identity = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        other_store(),
        std::num::NonZeroU64::new(7).unwrap(),
    );
    let scope = header_scope_known(other_identity);
    let (rejection, _) = rejection(
        crate::checkpoint_validation::record_matrix::Kind::Header,
        &HEADER,
        scope,
    );
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::StoreIdentityMismatch,
        field_range(scope, 16, 16),
        Some(worth_store_physical_integrity::PhysicalFormatField::StoreIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn footer_enforces_b_k_l_s_t_u_after_typed_prefix_validation() {
    let scope = footer_scope(identity());
    let mut covered_flip = FOOTER;
    covered_flip[40] ^= 1;
    assert_footer_damage(
        &covered_flip,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_flip = FOOTER;
    checksum_flip[155] ^= 1;
    assert_footer_damage(
        &checksum_flip,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut length_lie = FOOTER;
    length_lie[12..16].copy_from_slice(&1_u32.to_le_bytes());
    reseal_record(&mut length_lie);
    assert_footer_damage(
        &length_lie,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        field_range(scope, 12, 4),
        Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut identity_substitution = FOOTER;
    identity_substitution[32..40].copy_from_slice(&8_u64.to_le_bytes());
    reseal_record(&mut identity_substitution);
    assert_footer_damage(
        &identity_substitution,
        worth_store_physical_integrity::PhysicalDamageCause::SequenceMismatch,
        field_range(scope, 32, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::CheckpointIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    assert_footer_damage(
        &FOOTER[..155],
        worth_store_physical_integrity::PhysicalDamageCause::Truncated,
        worth_store_physical_integrity::PhysicalByteRange::new(
            scope.byte_range().end_exclusive() - 1,
            1,
        )
        .unwrap(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    // Schemas 2 and 3 are admitted; 4 remains outside the version window.
    let mut unsupported = FOOTER;
    unsupported[8] = 4;
    reseal_record(&mut unsupported);
    let (rejection, counters) = footer_rejection(&unsupported);
    let worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(version) =
        rejection
    else {
        panic!("unsupported footer schema collapsed into damage");
    };
    assert_eq!(
        version.axis(),
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::CheckpointRecordSchema
    );
    assert_eq!(version.observed(), 4);
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointFooter,
        unsupported.len() as u64,
        None,
    );
}
