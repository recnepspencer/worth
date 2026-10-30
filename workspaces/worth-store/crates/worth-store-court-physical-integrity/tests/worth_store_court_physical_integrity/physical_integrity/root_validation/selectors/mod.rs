#[allow(unused_imports)]
use crate::root_validation::selectors::*;

#[test]
fn clean_selector_controls_seal_typed_projections_and_exact_incarnation() {
    let store = store(7);
    let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);

    for kind in crate::support::root_validation::SelectorKind::ALL {
        let bytes = selector_bytes(kind, store, format);
        let other_incarnation = bytes;
        let scope = selector_scope(kind, store, format);
        let artifact =
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
        match kind {
            crate::support::root_validation::SelectorKind::Current => match validate_current_root_selector(artifact, scope) {
                (worth_store_physical_integrity::CurrentRootSelectorIntegrityValidation::Intact(validated), counters) => {
                    assert_eq!(validated.scope(), scope);
                    assert_eq!(validated.record_format(), format);
                    assert_eq!(validated.selector_identity().get(), 101);
                    assert_eq!(validated.root_generation(), 11);
                    assert_eq!(validated.linked_selector().unwrap().get(), 99);
                    assert_eq!(validated.linked_root_generation(), Some(10));
                    assert!(validated.matches_input(artifact));
                    assert!(!validated.matches_input(
                        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&other_incarnation)
                    ));
                    assert!(validated.into_validation_record().matches_scope(scope));
                    assert_intact_counters(counters, kind.family(), bytes.len() as u64);
                }
                (worth_store_physical_integrity::CurrentRootSelectorIntegrityValidation::Rejected(rejection), _) => {
                    panic!("clean current selector rejected: {rejection:?}")
                }
            },
            crate::support::root_validation::SelectorKind::Previous => match validate_previous_root_selector(artifact, scope) {
                (worth_store_physical_integrity::PreviousRootSelectorIntegrityValidation::Intact(validated), counters) => {
                    assert_eq!(validated.scope(), scope);
                    assert_eq!(validated.record_format(), format);
                    assert_eq!(validated.selector_identity().get(), 99);
                    assert_eq!(validated.root_generation(), 10);
                    assert_eq!(validated.linked_selector().unwrap().get(), 97);
                    assert_eq!(validated.linked_root_generation(), Some(9));
                    assert!(validated.matches_input(artifact));
                    assert!(!validated.matches_input(
                        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&other_incarnation)
                    ));
                    assert!(validated.into_validation_record().matches_scope(scope));
                    assert_intact_counters(counters, kind.family(), bytes.len() as u64);
                }
                (worth_store_physical_integrity::PreviousRootSelectorIntegrityValidation::Rejected(rejection), _) => {
                    panic!("clean previous selector rejected: {rejection:?}")
                }
            },
        }
    }
}

#[test]
fn selector_b_k_l_s_p_t_u_matrix_is_exact_for_both_roles() {
    let store = store(7);
    let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);
    for kind in crate::support::root_validation::SelectorKind::ALL {
        let scope = selector_scope(kind, store, format);

        let mut covered_byte_flip = selector_bytes(kind, store, format);
        covered_byte_flip[48] ^= 0x40;
        assert_selector_damage(
            kind,
            &covered_byte_flip,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        let mut checksum_flip = selector_bytes(kind, store, format);
        checksum_flip[44] ^= 0x01;
        assert_selector_damage(
            kind,
            &checksum_flip,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        let mut length_lie = selector_bytes(kind, store, format);
        length_lie[24..28].copy_from_slice(&60_u32.to_le_bytes());
        reseal_durable_frame(&mut length_lie);
        assert_selector_damage(
            kind,
            &length_lie,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
            field_range(scope, 20, 8),
            Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        let role_substitution = selector_bytes(kind.opposite(), store, format);
        assert_selector_damage(
            kind,
            &role_substitution,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::SelectorRoleMismatch,
            field_range(scope, 64, 1),
            Some(worth_store_physical_integrity::PhysicalFormatField::SelectorRole),
            worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
        );

        let mut pointer_corruption = selector_bytes(kind, store, format);
        pointer_corruption[81..89].copy_from_slice(&0_u64.to_le_bytes());
        reseal_durable_frame(&mut pointer_corruption);
        assert_selector_damage(
            kind,
            &pointer_corruption,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
            field_range(scope, 73, 16),
            Some(worth_store_physical_integrity::PhysicalFormatField::LinkedSelector),
            worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
        );

        let complete = selector_bytes(kind, store, format);
        let truncated = &complete[..104];
        assert_selector_damage(
            kind,
            truncated,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::Truncated,
            worth_store_physical_integrity::PhysicalByteRange::new(SELECTOR_OFFSET + 104, 3)
                .unwrap(),
            None,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );

        let mut unsupported_schema = selector_bytes(kind, store, format);
        unsupported_schema[9] = 3;
        reseal_durable_frame(&mut unsupported_schema);
        assert_unsupported(
            kind,
            &unsupported_schema,
            scope,
            worth_store_physical_integrity::PhysicalIntegrityVersionAxis::EnvelopeSchema,
            3,
        );

        let mut unsupported_envelope_format = selector_bytes(kind, store, format);
        unsupported_envelope_format[10..12].copy_from_slice(&3_u16.to_le_bytes());
        reseal_durable_frame(&mut unsupported_envelope_format);
        assert_unsupported(
            kind,
            &unsupported_envelope_format,
            scope,
            worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalFormat,
            3,
        );

        let mut unsupported_embedded_format = selector_bytes(kind, store, format);
        unsupported_embedded_format[89..91].copy_from_slice(&3_u16.to_le_bytes());
        reseal_durable_frame(&mut unsupported_embedded_format);
        assert_unsupported(
            kind,
            &unsupported_embedded_format,
            scope,
            worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalFormat,
            3,
        );
    }
}

#[test]
fn selector_scope_identity_generation_and_structure_denials_remain_distinct() {
    let store = store(7);
    let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);
    for kind in crate::support::root_validation::SelectorKind::ALL {
        let scope = selector_scope(kind, store, format);

        let other_store = selector_bytes(kind, crate::support::root_validation::store(8), format);
        assert_selector_damage(
            kind,
            &other_store,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::StoreIdentityMismatch,
            field_range(scope, 48, 16),
            Some(worth_store_physical_integrity::PhysicalFormatField::StoreIdentity),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );

        let mut zero_identity = selector_bytes(kind, store, format);
        zero_identity[28..36].copy_from_slice(&0_u64.to_le_bytes());
        reseal_durable_frame(&mut zero_identity);
        assert_selector_damage(
            kind,
            &zero_identity,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            field_range(scope, 28, 8),
            Some(worth_store_physical_integrity::PhysicalFormatField::ArtifactIdentity),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );

        let mut zero_generation = selector_bytes(kind, store, format);
        zero_generation[65..73].copy_from_slice(&0_u64.to_le_bytes());
        reseal_durable_frame(&mut zero_generation);
        assert_selector_damage(
            kind,
            &zero_generation,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            field_range(scope, 65, 8),
            Some(worth_store_physical_integrity::PhysicalFormatField::RootGeneration),
            worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
        );

        let mut family_substitution = selector_bytes(kind, store, format);
        family_substitution[8] = 2;
        reseal_durable_frame(&mut family_substitution);
        assert_selector_damage(
            kind,
            &family_substitution,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
            field_range(scope, 8, 1),
            Some(worth_store_physical_integrity::PhysicalFormatField::ArtifactFamily),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );

        let mut malformed = selector_bytes(kind, store, format);
        malformed[99] = 1;
        reseal_durable_frame(&mut malformed);
        assert_selector_damage(
            kind,
            &malformed,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
            field_range(scope, 99, 8),
            Some(worth_store_physical_integrity::PhysicalFormatField::Reserved),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );

        let mut malformed_embedded_format = selector_bytes(kind, store, format);
        malformed_embedded_format[95] = 9;
        reseal_durable_frame(&mut malformed_embedded_format);
        assert_selector_damage(
            kind,
            &malformed_embedded_format,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
            field_range(scope, 89, 10),
            Some(worth_store_physical_integrity::PhysicalFormatField::FormatDeclaration),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );

        let other_format = crate::support::root_validation::format(
            worth_store_physical_format::PhysicalPageSizeClass::KiB32,
        );
        let format_substitution = selector_bytes(kind, store, other_format);
        assert_selector_damage(
            kind,
            &format_substitution,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::FormatMismatch,
            field_range(scope, 10, 10),
            Some(worth_store_physical_integrity::PhysicalFormatField::FormatDeclaration),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

#[test]
fn selector_validators_reject_cross_role_and_manifest_scopes_before_interpretation() {
    let store = store(7);
    let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);
    for kind in crate::support::root_validation::SelectorKind::ALL {
        let bytes = selector_bytes(kind, store, format);
        for wrong_scope in [
            selector_scope(kind.opposite(), store, format),
            manifest_scope(store, format, 11),
        ] {
            let (rejection, counters) = validate_selector_rejection(kind, &bytes, wrong_scope);
            assert_damage(
                rejection,
                wrong_scope,
                worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
                wrong_scope.byte_range(),
                None,
                worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
            );
            assert_rejected_counters(
                counters,
                kind.family(),
                bytes.len() as u64,
                worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
            );
        }
    }
}
