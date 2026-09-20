use worth_foundational::facade::BoundaryProtocolCompatibilityWindow;
use worth_query_installation::facade::WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION;
use worth_query_package_archive::facade::*;

use super::archive_stream::fixture as package_fixture;
use super::release_envelope_fixture::{fixture_descriptor, signed_envelope_bytes};

const ENVELOPE_VERSION_OFFSET: usize = 8;
const ENVELOPE_BODY_LENGTH_OFFSET: usize = 10;
const ARCHIVE_VERSION_OFFSET: usize = 8;
const MANIFEST_PAYLOAD_LENGTH_OFFSET: usize = 10;
const MANIFEST_VERSION_OFFSET: usize = 14;
const MANIFEST_RECORD_COUNT_OFFSET: usize = 48;
const MANIFEST_FRAME_BYTES: usize = 118;
const RECORD_PAYLOAD_LENGTH_OFFSET: usize = MANIFEST_FRAME_BYTES + 8;

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

fn assert_exact_window(window: BoundaryProtocolCompatibilityWindow, expected: u16) {
    assert_eq!(window.earliest().get(), u32::from(expected));
    assert_eq!(window.latest().get(), u32::from(expected));
    assert_eq!(window.retired_before(), None);
}

fn assert_compatibility(
    denial: WorthQueryPackageArchiveDenial,
    expected_kind: WorthQueryPackageArchiveDenialKind,
    expected_layer: WorthQueryPackageArchiveProtocolLayer,
    expected_version: u16,
    expected_posture: WorthQueryPackageArchiveCompatibilityPosture,
) {
    assert_eq!(denial.kind(), expected_kind);
    let compatibility = denial.compatibility().unwrap();
    assert_eq!(compatibility.layer(), expected_layer);
    assert_eq!(compatibility.observed_version(), expected_version);
    assert_eq!(compatibility.posture(), expected_posture);
    assert_exact_window(
        compatibility.supported_window(),
        supported_version(expected_layer),
    );
}

fn supported_version(layer: WorthQueryPackageArchiveProtocolLayer) -> u16 {
    match layer {
        WorthQueryPackageArchiveProtocolLayer::ReleaseEnvelope => {
            WORTH_QUERY_PACKAGE_RELEASE_ENVELOPE_PROTOCOL_VERSION
        }
        WorthQueryPackageArchiveProtocolLayer::Archive => {
            WORTH_QUERY_PACKAGE_ARCHIVE_PROTOCOL_VERSION
        }
        WorthQueryPackageArchiveProtocolLayer::Manifest => {
            WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION.get()
        }
        WorthQueryPackageArchiveProtocolLayer::RecordFrame => {
            WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION
        }
        WorthQueryPackageArchiveProtocolLayer::ApplicationProgramDescription => {
            WORTH_QUERY_APPLICATION_PROGRAM_ARCHIVE_PROTOCOL_VERSION
        }
    }
}

fn assert_layer(
    denial: WorthQueryPackageArchiveDenial,
    expected_layer: WorthQueryPackageArchiveProtocolLayer,
) {
    let compatibility = denial.compatibility().unwrap();
    assert_eq!(compatibility.layer(), expected_layer);
    assert_eq!(
        compatibility.posture(),
        WorthQueryPackageArchiveCompatibilityPosture::ExceedsWindow
    );
}

fn release_envelope_bytes() -> Vec<u8> {
    let records = package_fixture::minimal_package()
        .export_typed_records()
        .unwrap();
    signed_envelope_bytes(&records, fixture_descriptor())
}

fn package_archive_bytes() -> Vec<u8> {
    let records = package_fixture::minimal_package()
        .export_typed_records()
        .unwrap();
    encode_package_archive(&records, WorthQueryPackageArchiveLimits::DEFAULT).unwrap()
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_be_bytes());
}
