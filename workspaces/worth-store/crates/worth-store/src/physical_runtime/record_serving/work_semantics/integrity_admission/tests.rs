use super::*;

pub(in crate::physical_runtime) fn assert_actual_lifecycle_denial_maps_without_damage(
    denial: ResidentIntegrityAdmissionDenial,
) {
    let inline = classify_inline_integrity(denial);
    let extent = classify_extent_integrity(denial);
    assert_eq!(inline, CleanInlineAdmissionDenial::RuntimeReleased);
    assert_eq!(extent, CleanExtentAdmissionDenial::RuntimeReleased);
    assert!(inline.preserves_resident_bytes());
    assert!(extent.preserves_resident_bytes());
    assert_eq!(
        inline.read_denial(),
        crate::physical_runtime::record_serving::RecordReadDenial::PhysicalWork(
            crate::physical_runtime::record_serving::RecordReadWorkDenial::RuntimeReleased
        )
    );
    assert_eq!(
        extent.read_denial(),
        crate::physical_runtime::record_serving::RecordReadDenial::PhysicalWork(
            crate::physical_runtime::record_serving::RecordReadWorkDenial::RuntimeReleased
        )
    );
    assert_eq!(
        extent.stream_failure_kind(),
        crate::physical_runtime::record_serving::RecordStreamFailureKind::RuntimeReleased
    );
}

#[test]
fn proof_lifetime_denials_are_unavailable_without_accusing_resident_bytes() {
    for denial in [
        ResidentIntegrityAdmissionDenial::SourceIncarnationMismatch,
        ResidentIntegrityAdmissionDenial::RetainedRecordInvalidated,
        ResidentIntegrityAdmissionDenial::RetainedRecordChanged,
    ] {
        let inline = classify_inline_integrity(denial);
        let extent = classify_extent_integrity(denial);
        assert_eq!(inline, CleanInlineAdmissionDenial::Unavailable);
        assert_eq!(extent, CleanExtentAdmissionDenial::Unavailable);
        assert!(inline.preserves_resident_bytes());
        assert!(extent.preserves_resident_bytes());
        assert_eq!(
            inline.read_denial(),
            crate::physical_runtime::record_serving::RecordReadDenial::ArtifactUnavailable
        );
        assert_eq!(
            extent.read_denial(),
            crate::physical_runtime::record_serving::RecordReadDenial::ArtifactUnavailable
        );
        assert_eq!(
            extent.stream_failure_kind(),
            crate::physical_runtime::record_serving::RecordStreamFailureKind::ArtifactUnavailable
        );
        assert!(denial.preserves_resident_bytes());
        assert_eq!(denial.residency_unavailability(), None);
    }
}

#[test]
fn validated_damage_and_unsupported_format_remain_distinct() {
    use worth_store_physical_integrity::{
        PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalDamageLocalization,
        PhysicalIntegrityRejection, PhysicalIntegrityVersionAxis,
        UnsupportedPhysicalIntegrityVersion,
    };

    let scope = fixture_scope();
    let range = PhysicalByteRange::new(0, 64).unwrap();
    let damaged = ResidentIntegrityAdmissionDenial::Validation(
        PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            range,
            None,
            PhysicalBlastRadius::CanonicalFrame,
        )),
    );
    let unsupported = ResidentIntegrityAdmissionDenial::Validation(
        PhysicalIntegrityRejection::Unsupported(UnsupportedPhysicalIntegrityVersion::new(
            scope,
            PhysicalIntegrityVersionAxis::PhysicalFormat,
            99,
        )),
    );

    assert_eq!(
        classify_inline_integrity(damaged),
        CleanInlineAdmissionDenial::Damaged
    );
    assert_eq!(
        classify_extent_integrity(damaged),
        CleanExtentAdmissionDenial::Damaged
    );
    assert_eq!(
        classify_inline_integrity(unsupported),
        CleanInlineAdmissionDenial::Format
    );
    assert_eq!(
        classify_extent_integrity(unsupported),
        CleanExtentAdmissionDenial::Format
    );
    assert_eq!(
        classify_inline_integrity(damaged).read_denial(),
        crate::physical_runtime::record_serving::RecordReadDenial::ArtifactDamaged
    );
    assert_eq!(
        classify_extent_integrity(unsupported).stream_failure_kind(),
        crate::physical_runtime::record_serving::RecordStreamFailureKind::FormatMismatch
    );
}

#[test]
fn frame_generation_change_preserves_the_exact_residency_form() {
    let denial = ResidentIntegrityAdmissionDenial::FrameGenerationChanged;
    let expected = worth_store_buffer_pool::PhysicalResidencyDenial::FrameNotResident;
    let inline = classify_inline_integrity(denial);
    let extent = classify_extent_integrity(denial);

    assert_eq!(inline, CleanInlineAdmissionDenial::Residency(expected));
    assert_eq!(extent, CleanExtentAdmissionDenial::Residency(expected));
    assert!(inline.preserves_resident_bytes());
    assert!(extent.preserves_resident_bytes());
    assert_eq!(
        extent.stream_failure_kind(),
        crate::physical_runtime::record_serving::RecordStreamFailureKind::ResidencyUnavailable(
            expected.into()
        )
    );
}

fn fixture_scope() -> worth_store_physical_integrity::PhysicalArtifactScope {
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };

    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([87; 16]).unwrap(),
    )
    .published_identity();
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    worth_store_physical_integrity::PhysicalArtifactScope::root_manifest(
        store,
        format,
        1,
        worth_store_physical_integrity::PhysicalByteRange::new(0, 64).unwrap(),
    )
    .unwrap()
}
