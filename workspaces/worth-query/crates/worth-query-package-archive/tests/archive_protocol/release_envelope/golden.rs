//! Current envelope golden and retained unsupported inner-record vectors.
use super::*;

const RECORD_SIX_ENVELOPE_HEX: &str = include_str!("release_envelope_v1.hex");
const RECORD_SEVEN_ENVELOPE_HEX: &str = include_str!("release_envelope_v1_record_v7.hex");
const RECORD_NINE_ENVELOPE_HEX: &str = include_str!("release_envelope_v1_record_v9.hex");
const RECORD_TEN_ENVELOPE_HEX: &str = include_str!("release_envelope_v1_record_v10.hex");
const RECORD_TWELVE_ENVELOPE_HEX: &str = include_str!("release_envelope_v1_record_v12.hex");
const RECORD_THIRTEEN_ENVELOPE_HEX: &str = include_str!("release_envelope_v1_record_v13.hex");
const RECORD_FOURTEEN_ENVELOPE_HEX: &str = include_str!("release_envelope_v1_record_v14.hex");
const CURRENT_ENVELOPE_HEX: &str = include_str!("release_envelope_v1_record_v16.hex");

pub(super) fn assert_current_envelope_and_unsupported_records() {
    for (version, vector) in [
        (6_u16, RECORD_SIX_ENVELOPE_HEX),
        (7_u16, RECORD_SEVEN_ENVELOPE_HEX),
        (9_u16, RECORD_NINE_ENVELOPE_HEX),
        (10_u16, RECORD_TEN_ENVELOPE_HEX),
        (12_u16, RECORD_TWELVE_ENVELOPE_HEX),
        (13_u16, RECORD_THIRTEEN_ENVELOPE_HEX),
        (14_u16, RECORD_FOURTEEN_ENVELOPE_HEX),
    ] {
        let original = decode_hex(vector.trim());
        // A valid outer envelope/checksum does not admit an unsupported inner record.
        let untrusted =
            decode_package_release_envelope(&original, WorthQueryPackageEnvelopeLimits::DEFAULT)
                .unwrap();
        let denial =
            decode_package_archive(untrusted.archive(), WorthQueryPackageArchiveLimits::DEFAULT)
                .unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
        );
        assert_eq!(denial.compatibility().unwrap().observed_version(), version);
    }

    let records = fixture::minimal_package().export_typed_records().unwrap();
    let first = signed_envelope_bytes(&records, fixture_descriptor());
    assert_eq!(signed_envelope_bytes(&records, fixture_descriptor()), first);
    assert_eq!(encode_hex(&first), CURRENT_ENVELOPE_HEX.trim());

    let frozen = decode_hex(CURRENT_ENVELOPE_HEX.trim());
    assert_eq!(
        u16::from_be_bytes(frozen[8..10].try_into().unwrap()),
        1,
        "outer envelope version stays1"
    );
    let decoded =
        decode_package_release_envelope(&frozen, WorthQueryPackageEnvelopeLimits::DEFAULT).unwrap();
    assert_eq!(decoded.signature(), &[0xa5; 64]);
    assert_eq!(
        decoded.expected_package_identity(),
        records.manifest().package_identity()
    );
    assert_eq!(
        u16::from_be_bytes(decoded.archive()[8..10].try_into().unwrap()),
        2
    );
    assert_eq!(
        u16::from_be_bytes(decoded.archive()[118..120].try_into().unwrap()),
        16
    );
    let archive =
        decode_package_archive(decoded.archive(), WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    assert_eq!(archive.manifest(), records.manifest());
    assert_eq!(archive.frames().len(), records.records().len());
    for (frame, expected) in archive.frames().iter().zip(records.records()) {
        assert_eq!(frame.record(), expected);
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut encoded, byte| {
        write!(&mut encoded, "{byte:02x}").unwrap();
        encoded
    })
}

fn decode_hex(encoded: &str) -> Vec<u8> {
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digits = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(digits, 16).unwrap()
        })
        .collect()
}
