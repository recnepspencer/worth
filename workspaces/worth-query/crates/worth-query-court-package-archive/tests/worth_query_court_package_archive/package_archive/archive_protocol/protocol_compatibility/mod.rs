pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::protocol_compatibility::*;

#[test]
fn current_reader_profile_is_derived_from_every_public_version_constant() {
    let profile = WorthQueryPackageArchiveCompatibilityProfile::CURRENT;
    assert_exact_window(
        profile.release_envelope_window(),
        WORTH_QUERY_PACKAGE_RELEASE_ENVELOPE_PROTOCOL_VERSION,
    );
    assert_exact_window(
        profile.archive_window(),
        WORTH_QUERY_PACKAGE_ARCHIVE_PROTOCOL_VERSION,
    );
    assert_exact_window(
        profile.manifest_window(),
        WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION.get(),
    );
    assert_exact_window(
        profile.record_frame_window(),
        WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION,
    );
    assert_exact_window(
        profile.application_program_description_window(),
        WORTH_QUERY_APPLICATION_PROGRAM_ARCHIVE_PROTOCOL_VERSION,
    );
    assert_window(
        profile.workflow_definition_draft_window(),
        7,
        WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION,
    );
}

#[test]
fn zero_and_future_versions_report_the_exact_protocol_layer_and_posture() {
    for zero in [true, false] {
        let posture = if zero {
            WorthQueryPackageArchiveCompatibilityPosture::InvalidZero
        } else {
            WorthQueryPackageArchiveCompatibilityPosture::ExceedsWindow
        };
        let observed = |current: u16| if zero { 0 } else { current + 1 };
        let mut envelope = release_envelope_bytes();
        let observed = observed(WORTH_QUERY_PACKAGE_RELEASE_ENVELOPE_PROTOCOL_VERSION);
        put_u16(&mut envelope, ENVELOPE_VERSION_OFFSET, observed);
        assert_compatibility(
            decode_package_release_envelope(&envelope, WorthQueryPackageEnvelopeLimits::DEFAULT)
                .unwrap_err(),
            WorthQueryPackageArchiveDenialKind::UnsupportedEnvelopeVersion,
            WorthQueryPackageArchiveProtocolLayer::ReleaseEnvelope,
            observed,
            posture,
        );

        let mut archive = package_archive_bytes();
        let observed = if zero {
            0
        } else {
            WORTH_QUERY_PACKAGE_ARCHIVE_PROTOCOL_VERSION + 1
        };
        put_u16(&mut archive, ARCHIVE_VERSION_OFFSET, observed);
        assert_compatibility(
            decode_package_archive(&archive, WorthQueryPackageArchiveLimits::DEFAULT).unwrap_err(),
            WorthQueryPackageArchiveDenialKind::UnsupportedArchiveVersion,
            WorthQueryPackageArchiveProtocolLayer::Archive,
            observed,
            posture,
        );

        let mut manifest = package_archive_bytes();
        let observed = if zero {
            0
        } else {
            WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION.get() + 1
        };
        put_u16(&mut manifest, MANIFEST_VERSION_OFFSET, observed);
        assert_compatibility(
            decode_package_archive(&manifest, WorthQueryPackageArchiveLimits::DEFAULT).unwrap_err(),
            WorthQueryPackageArchiveDenialKind::UnsupportedManifestVersion,
            WorthQueryPackageArchiveProtocolLayer::Manifest,
            observed,
            posture,
        );

        let mut record = package_archive_bytes();
        let observed = if zero {
            0
        } else {
            WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION + 1
        };
        put_u16(&mut record, MANIFEST_FRAME_BYTES, observed);
        assert_compatibility(
            decode_package_archive(&record, WorthQueryPackageArchiveLimits::DEFAULT).unwrap_err(),
            WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion,
            WorthQueryPackageArchiveProtocolLayer::RecordFrame,
            observed,
            posture,
        );
    }
}

#[test]
fn displaced_correction_record_grammars_are_refused_before_payload_decode() {
    for version in [10_u16, 11_u16, 12_u16] {
        let mut archive = package_archive_bytes();
        put_u16(&mut archive, MANIFEST_FRAME_BYTES, version);
        let denial =
            decode_package_archive(&archive, WorthQueryPackageArchiveLimits::DEFAULT).unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
        );
        assert_eq!(denial.compatibility().unwrap().observed_version(), version);
    }
}

#[test]
fn unsupported_headers_win_before_hostile_body_claims() {
    let mut envelope = release_envelope_bytes();
    put_u16(
        &mut envelope,
        ENVELOPE_VERSION_OFFSET,
        WORTH_QUERY_PACKAGE_RELEASE_ENVELOPE_PROTOCOL_VERSION + 1,
    );
    put_u64(&mut envelope, ENVELOPE_BODY_LENGTH_OFFSET, u64::MAX);
    assert_layer(
        decode_package_release_envelope(&envelope, WorthQueryPackageEnvelopeLimits::DEFAULT)
            .unwrap_err(),
        WorthQueryPackageArchiveProtocolLayer::ReleaseEnvelope,
    );

    let mut archive = package_archive_bytes();
    put_u16(
        &mut archive,
        ARCHIVE_VERSION_OFFSET,
        WORTH_QUERY_PACKAGE_ARCHIVE_PROTOCOL_VERSION + 1,
    );
    put_u32(&mut archive, MANIFEST_PAYLOAD_LENGTH_OFFSET, u32::MAX);
    assert_layer(
        decode_package_archive(&archive, WorthQueryPackageArchiveLimits::DEFAULT).unwrap_err(),
        WorthQueryPackageArchiveProtocolLayer::Archive,
    );

    let mut manifest = package_archive_bytes();
    put_u16(
        &mut manifest,
        MANIFEST_VERSION_OFFSET,
        WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION.get() + 1,
    );
    put_u32(&mut manifest, MANIFEST_RECORD_COUNT_OFFSET, u32::MAX);
    assert_layer(
        decode_package_archive(&manifest, WorthQueryPackageArchiveLimits::DEFAULT).unwrap_err(),
        WorthQueryPackageArchiveProtocolLayer::Manifest,
    );

    let mut record = package_archive_bytes();
    put_u16(
        &mut record,
        MANIFEST_FRAME_BYTES,
        WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION + 1,
    );
    put_u32(&mut record, RECORD_PAYLOAD_LENGTH_OFFSET, u32::MAX);
    assert_layer(
        decode_package_archive(&record, WorthQueryPackageArchiveLimits::DEFAULT).unwrap_err(),
        WorthQueryPackageArchiveProtocolLayer::RecordFrame,
    );
}
