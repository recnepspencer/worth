pub use worth_foundational::facade::BoundaryProtocolCompatibilityWindow;
pub use worth_query_installation::facade::WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION;
pub use worth_query_package_archive::facade::*;

pub use super::archive_stream::fixture as package_fixture;
pub use super::release_envelope_fixture::{fixture_descriptor, signed_envelope_bytes};

pub const ENVELOPE_VERSION_OFFSET: usize = 8;
pub const ENVELOPE_BODY_LENGTH_OFFSET: usize = 10;
pub const ARCHIVE_VERSION_OFFSET: usize = 8;
pub const MANIFEST_PAYLOAD_LENGTH_OFFSET: usize = 10;
pub const MANIFEST_VERSION_OFFSET: usize = 14;
pub const MANIFEST_RECORD_COUNT_OFFSET: usize = 48;
pub const MANIFEST_FRAME_BYTES: usize = 118;
pub const RECORD_PAYLOAD_LENGTH_OFFSET: usize = MANIFEST_FRAME_BYTES + 8;

pub fn assert_exact_window(window: BoundaryProtocolCompatibilityWindow, expected: u16) {
    assert_window(window, expected, expected);
}

pub fn assert_window(window: BoundaryProtocolCompatibilityWindow, earliest: u16, latest: u16) {
    assert_eq!(window.earliest().get(), u32::from(earliest));
    assert_eq!(window.latest().get(), u32::from(latest));
    assert_eq!(window.retired_before(), None);
}

pub fn assert_compatibility(
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
    let (earliest, latest) = supported_window(expected_layer);
    assert_window(compatibility.supported_window(), earliest, latest);
}

pub fn supported_window(layer: WorthQueryPackageArchiveProtocolLayer) -> (u16, u16) {
    let current = match layer {
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
        WorthQueryPackageArchiveProtocolLayer::WorkflowDefinitionDraft => {
            WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION
        }
    };
    if layer == WorthQueryPackageArchiveProtocolLayer::WorkflowDefinitionDraft {
        (7, current)
    } else {
        (current, current)
    }
}

pub fn assert_layer(
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

pub fn release_envelope_bytes() -> Vec<u8> {
    let records = package_fixture::minimal_package()
        .export_typed_records()
        .unwrap();
    signed_envelope_bytes(&records, fixture_descriptor())
}

pub fn package_archive_bytes() -> Vec<u8> {
    let records = package_fixture::minimal_package()
        .export_typed_records()
        .unwrap();
    encode_package_archive(&records, WorthQueryPackageArchiveLimits::DEFAULT).unwrap()
}

pub fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

pub fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

pub fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_be_bytes());
}
