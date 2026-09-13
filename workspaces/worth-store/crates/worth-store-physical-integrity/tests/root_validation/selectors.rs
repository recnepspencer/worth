use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::PhysicalPageSizeClass;
use worth_store_physical_integrity::{
    validate_current_root_selector, validate_previous_root_selector,
    CurrentRootSelectorIntegrityValidation, PhysicalBlastRadius, PhysicalByteRange,
    PhysicalDamageCause, PhysicalFormatField, PhysicalIntegrityRejection,
    PhysicalIntegrityRejectionClass, PhysicalIntegrityVersionAxis,
    PreviousRootSelectorIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, assert_rejected_counters, field_range, format, manifest_scope,
    reseal_durable_frame, selector_bytes, selector_scope, store, validate_selector_rejection,
    SelectorKind, SELECTOR_OFFSET,
};

#[test]
fn clean_selector_controls_seal_typed_projections_and_exact_incarnation() {
    let store = store(7);
    let format = format(PhysicalPageSizeClass::KiB16);

    for kind in SelectorKind::ALL {
        let bytes = selector_bytes(kind, store, format);
        let other_incarnation = bytes;
        let scope = selector_scope(kind, store, format);
        let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
        match kind {
            SelectorKind::Current => match validate_current_root_selector(artifact, scope) {
                (CurrentRootSelectorIntegrityValidation::Intact(validated), counters) => {
                    assert_eq!(validated.scope(), scope);
                    assert_eq!(validated.record_format(), format);
                    assert_eq!(validated.selector_identity().get(), 101);
                    assert_eq!(validated.root_generation(), 11);
                    assert_eq!(validated.linked_selector().unwrap().get(), 99);
                    assert_eq!(validated.linked_root_generation(), Some(10));
                    assert!(validated.matches_input(artifact));
                    assert!(!validated.matches_input(
                        UntrustedPhysicalArtifact::from_bounded_bytes(&other_incarnation)
                    ));
                    assert!(validated.into_validation_record().matches_scope(scope));
                    assert_intact_counters(counters, kind.family(), bytes.len() as u64);
                }
                (CurrentRootSelectorIntegrityValidation::Rejected(rejection), _) => {
                    panic!("clean current selector rejected: {rejection:?}")
                }
            },
            SelectorKind::Previous => match validate_previous_root_selector(artifact, scope) {
                (PreviousRootSelectorIntegrityValidation::Intact(validated), counters) => {
                    assert_eq!(validated.scope(), scope);
                    assert_eq!(validated.record_format(), format);
                    assert_eq!(validated.selector_identity().get(), 99);
                    assert_eq!(validated.root_generation(), 10);
                    assert_eq!(validated.linked_selector().unwrap().get(), 97);
                    assert_eq!(validated.linked_root_generation(), Some(9));
                    assert!(validated.matches_input(artifact));
                    assert!(!validated.matches_input(
                        UntrustedPhysicalArtifact::from_bounded_bytes(&other_incarnation)
                    ));
                    assert!(validated.into_validation_record().matches_scope(scope));
                    assert_intact_counters(counters, kind.family(), bytes.len() as u64);
                }
                (PreviousRootSelectorIntegrityValidation::Rejected(rejection), _) => {
                    panic!("clean previous selector rejected: {rejection:?}")
                }
            },
        }
    }
}

#[test]
fn selector_b_k_l_s_p_t_u_matrix_is_exact_for_both_roles() {
    let store = store(7);
    let format = format(PhysicalPageSizeClass::KiB16);
    for kind in SelectorKind::ALL {
        let scope = selector_scope(kind, store, format);

        let mut covered_byte_flip = selector_bytes(kind, store, format);
        covered_byte_flip[48] ^= 0x40;
        assert_selector_damage(
            kind,
            &covered_byte_flip,
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        );

        let mut checksum_flip = selector_bytes(kind, store, format);
        checksum_flip[44] ^= 0x01;
        assert_selector_damage(
            kind,
            &checksum_flip,
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        );

        let mut length_lie = selector_bytes(kind, store, format);
        length_lie[24..28].copy_from_slice(&60_u32.to_le_bytes());
        reseal_durable_frame(&mut length_lie);
        assert_selector_damage(
            kind,
            &length_lie,
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            field_range(scope, 20, 8),
            Some(PhysicalFormatField::EncodedLength),
            PhysicalBlastRadius::CanonicalFrame,
        );

        let role_substitution = selector_bytes(kind.opposite(), store, format);
        assert_selector_damage(
            kind,
            &role_substitution,
            scope,
            PhysicalDamageCause::SelectorRoleMismatch,
            field_range(scope, 64, 1),
            Some(PhysicalFormatField::SelectorRole),
            PhysicalBlastRadius::ReachableSubtree,
        );

        let mut pointer_corruption = selector_bytes(kind, store, format);
        pointer_corruption[81..89].copy_from_slice(&0_u64.to_le_bytes());
        reseal_durable_frame(&mut pointer_corruption);
        assert_selector_damage(
            kind,
            &pointer_corruption,
            scope,
            PhysicalDamageCause::ChildReferenceMismatch,
            field_range(scope, 73, 16),
            Some(PhysicalFormatField::LinkedSelector),
            PhysicalBlastRadius::ReachableSubtree,
        );

        let complete = selector_bytes(kind, store, format);
        let truncated = &complete[..104];
        assert_selector_damage(
            kind,
            truncated,
            scope,
            PhysicalDamageCause::Truncated,
            PhysicalByteRange::new(SELECTOR_OFFSET + 104, 3).unwrap(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        );

        let mut unsupported_schema = selector_bytes(kind, store, format);
        unsupported_schema[9] = 3;
        reseal_durable_frame(&mut unsupported_schema);
        assert_unsupported(
            kind,
            &unsupported_schema,
            scope,
            PhysicalIntegrityVersionAxis::EnvelopeSchema,
            3,
        );

        let mut unsupported_envelope_format = selector_bytes(kind, store, format);
        unsupported_envelope_format[10..12].copy_from_slice(&2_u16.to_le_bytes());
        reseal_durable_frame(&mut unsupported_envelope_format);
        assert_unsupported(
            kind,
            &unsupported_envelope_format,
            scope,
            PhysicalIntegrityVersionAxis::PhysicalFormat,
            2,
        );

        let mut unsupported_embedded_format = selector_bytes(kind, store, format);
        unsupported_embedded_format[89..91].copy_from_slice(&2_u16.to_le_bytes());
        reseal_durable_frame(&mut unsupported_embedded_format);
        assert_unsupported(
            kind,
            &unsupported_embedded_format,
            scope,
            PhysicalIntegrityVersionAxis::PhysicalFormat,
            2,
        );
    }
}

#[test]
fn selector_scope_identity_generation_and_structure_denials_remain_distinct() {
    let store = store(7);
    let format = format(PhysicalPageSizeClass::KiB16);
    for kind in SelectorKind::ALL {
        let scope = selector_scope(kind, store, format);

        let other_store = selector_bytes(kind, super::support::store(8), format);
        assert_selector_damage(
            kind,
            &other_store,
            scope,
            PhysicalDamageCause::StoreIdentityMismatch,
            field_range(scope, 48, 16),
            Some(PhysicalFormatField::StoreIdentity),
            PhysicalBlastRadius::CompleteArtifact,
        );

        let mut zero_identity = selector_bytes(kind, store, format);
        zero_identity[28..36].copy_from_slice(&0_u64.to_le_bytes());
        reseal_durable_frame(&mut zero_identity);
        assert_selector_damage(
            kind,
            &zero_identity,
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            field_range(scope, 28, 8),
            Some(PhysicalFormatField::ArtifactIdentity),
            PhysicalBlastRadius::CompleteArtifact,
        );

        let mut zero_generation = selector_bytes(kind, store, format);
        zero_generation[65..73].copy_from_slice(&0_u64.to_le_bytes());
        reseal_durable_frame(&mut zero_generation);
        assert_selector_damage(
            kind,
            &zero_generation,
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            field_range(scope, 65, 8),
            Some(PhysicalFormatField::RootGeneration),
            PhysicalBlastRadius::ReachableSubtree,
        );

        let mut family_substitution = selector_bytes(kind, store, format);
        family_substitution[8] = 2;
        reseal_durable_frame(&mut family_substitution);
        assert_selector_damage(
            kind,
            &family_substitution,
            scope,
            PhysicalDamageCause::FamilyMismatch,
            field_range(scope, 8, 1),
            Some(PhysicalFormatField::ArtifactFamily),
            PhysicalBlastRadius::CompleteArtifact,
        );

        let mut malformed = selector_bytes(kind, store, format);
        malformed[99] = 1;
        reseal_durable_frame(&mut malformed);
        assert_selector_damage(
            kind,
            &malformed,
            scope,
            PhysicalDamageCause::MalformedStructure,
            field_range(scope, 99, 8),
            Some(PhysicalFormatField::Reserved),
            PhysicalBlastRadius::CompleteArtifact,
        );

        let mut malformed_embedded_format = selector_bytes(kind, store, format);
        malformed_embedded_format[95] = 9;
        reseal_durable_frame(&mut malformed_embedded_format);
        assert_selector_damage(
            kind,
            &malformed_embedded_format,
            scope,
            PhysicalDamageCause::MalformedStructure,
            field_range(scope, 89, 10),
            Some(PhysicalFormatField::FormatDeclaration),
            PhysicalBlastRadius::CompleteArtifact,
        );

        let other_format = super::support::format(PhysicalPageSizeClass::KiB32);
        let format_substitution = selector_bytes(kind, store, other_format);
        assert_selector_damage(
            kind,
            &format_substitution,
            scope,
            PhysicalDamageCause::FormatMismatch,
            field_range(scope, 10, 10),
            Some(PhysicalFormatField::FormatDeclaration),
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

#[test]
fn selector_validators_reject_cross_role_and_manifest_scopes_before_interpretation() {
    let store = store(7);
    let format = format(PhysicalPageSizeClass::KiB16);
    for kind in SelectorKind::ALL {
        let bytes = selector_bytes(kind, store, format);
        for wrong_scope in [
            selector_scope(kind.opposite(), store, format),
            manifest_scope(store, format, 11),
        ] {
            let (rejection, counters) = validate_selector_rejection(kind, &bytes, wrong_scope);
            assert_damage(
                rejection,
                wrong_scope,
                PhysicalDamageCause::FamilyMismatch,
                wrong_scope.byte_range(),
                None,
                PhysicalBlastRadius::CompleteArtifact,
            );
            assert_rejected_counters(
                counters,
                kind.family(),
                bytes.len() as u64,
                PhysicalDamageCause::FamilyMismatch,
            );
        }
    }
}

fn assert_selector_damage(
    kind: SelectorKind,
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (rejection, counters) = validate_selector_rejection(kind, bytes, scope);
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(counters, kind.family(), bytes.len() as u64, cause);
}

fn assert_unsupported(
    kind: SelectorKind,
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (rejection, counters) = validate_selector_rejection(kind, bytes, scope);
    let PhysicalIntegrityRejection::Unsupported(unsupported) = rejection else {
        panic!("expected unsupported selector version, got {rejection:?}");
    };
    assert_eq!(unsupported.scope(), scope);
    assert_eq!(unsupported.axis(), axis);
    assert_eq!(unsupported.observed(), observed);
    assert_eq!(counters.family(), kind.family());
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), bytes.len() as u64);
    assert_eq!(counters.intact_frames(), 0);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(PhysicalIntegrityRejectionClass::Unsupported),
        1
    );
}

fn assert_intact_counters(
    counters: worth_store_physical_integrity::PhysicalIntegrityObservationCounters,
    family: PhysicalIntegrityArtifactFamily,
    byte_count: u64,
) {
    assert_eq!(counters.family(), family);
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), byte_count);
    assert_eq!(counters.intact_frames(), 1);
    assert_eq!(counters.rejected_frames(), 0);
}
