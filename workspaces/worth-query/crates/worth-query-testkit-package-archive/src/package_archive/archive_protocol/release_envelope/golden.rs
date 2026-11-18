//! Current envelope golden and retained unsupported inner-record vectors.
pub use super::*;

// D1289 canonical policy v2; the envelope/record framing is unchanged.
pub const CURRENT_ENVELOPE_HEX: &str = include_str!("release_envelope_v1_record_v19.hex");

pub fn assert_current_envelope_and_unsupported_records() {
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
        19
    );
    let archive =
        decode_package_archive(decoded.archive(), WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    assert_eq!(archive.manifest(), records.manifest());
    assert_eq!(archive.frames().len(), records.records().len());
    for (frame, expected) in archive.frames().iter().zip(records.records()) {
        assert_eq!(frame.record(), expected);
    }
}

pub fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut encoded, byte| {
        write!(&mut encoded, "{byte:02x}").unwrap();
        encoded
    })
}

pub fn decode_hex(encoded: &str) -> Vec<u8> {
    encoded
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let digits = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(digits, 16).unwrap()
        })
        .collect()
}
