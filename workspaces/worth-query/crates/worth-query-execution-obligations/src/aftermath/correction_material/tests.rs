//! Pure envelope grammar; no Home owner or correction authority is fabricated.
use super::*;
use sha2::{Digest, Sha256};
fn frame(class: RetentionClass) -> QueryCorrectionMaterialFrame {
    QueryCorrectionMaterialFrame::prepare(
        HomeIdentity::from_observed_bytes([1; 16]).unwrap(),
        NonZeroU64::MIN,
        [0; 32],
        NonZeroU64::MIN,
        class,
        b"recorded-body",
    )
    .unwrap()
}
fn with_checksum(mut bytes: Vec<u8>) -> Vec<u8> {
    let end = bytes.len() - 32;
    let digest = Sha256::digest(&bytes[..end]);
    bytes[end..].copy_from_slice(&digest);
    bytes
}
#[test]
fn correction_envelope_preserves_exact_body_class_and_positioned_coordinates() {
    for class in [
        RetentionClass::Authoritative,
        RetentionClass::MandatoryObligation,
        RetentionClass::OptionalRetainedMaterial,
        RetentionClass::Derived,
    ] {
        let original = frame(class);
        let read = QueryCorrectionMaterialFrame::readmit(original.bytes().to_vec()).unwrap();
        assert_eq!(read, original);
        assert_eq!(read.body(), b"recorded-body");
        assert_eq!(read.position(), 1);
        assert_eq!(read.predecessor(), [0; 32]);
        assert_eq!(read.publication_generation(), NonZeroU64::MIN);
        assert_eq!(read.body_retention_class(), class);
        assert_eq!(&read.bytes()[..8], b"WQCOR001");
        assert_eq!(
            read.payload_digest(),
            <[u8; 32]>::from(Sha256::digest(read.bytes()))
        );
    }
}
#[test]
fn correction_envelope_refuses_truncation_corruption_and_trailing_body_bytes() {
    let original = frame(RetentionClass::OptionalRetainedMaterial);
    for length in 0..original.bytes().len() {
        assert!(
            QueryCorrectionMaterialFrame::readmit(original.bytes()[..length].to_vec()).is_err()
        );
    }
    let mut bytes = original.bytes().to_vec();
    bytes[framing::HEADER] ^= 1;
    assert_eq!(
        QueryCorrectionMaterialFrame::readmit(bytes),
        Err(QueryCorrectionMaterialDenial::Checksum)
    );
    let mut bytes = original.bytes().to_vec();
    bytes.insert(bytes.len() - 32, 0);
    assert_eq!(
        QueryCorrectionMaterialFrame::readmit(with_checksum(bytes)),
        Err(QueryCorrectionMaterialDenial::MalformedFrame)
    );
}
#[test]
fn correction_envelope_refuses_unsupported_version_zero_coordinates_and_unknown_class() {
    let original = frame(RetentionClass::MandatoryObligation);
    let mut version = original.bytes().to_vec();
    version[8..12].copy_from_slice(&9u32.to_le_bytes());
    assert_eq!(
        QueryCorrectionMaterialFrame::readmit(with_checksum(version)),
        Err(QueryCorrectionMaterialDenial::UnsupportedVersion(9))
    );
    for range in [12..28, 28..36, 68..76] {
        let mut bytes = original.bytes().to_vec();
        bytes[range].fill(0);
        assert_eq!(
            QueryCorrectionMaterialFrame::readmit(with_checksum(bytes)),
            Err(QueryCorrectionMaterialDenial::MalformedFrame)
        );
    }
    let mut bytes = original.bytes().to_vec();
    bytes[76] = 255;
    assert_eq!(
        QueryCorrectionMaterialFrame::readmit(with_checksum(bytes)),
        Err(QueryCorrectionMaterialDenial::MalformedFrame)
    );
}
#[test]
fn correction_envelope_never_allocates_from_an_untrusted_declared_body_length() {
    let mut bytes = frame(RetentionClass::OptionalRetainedMaterial)
        .bytes()
        .to_vec();
    bytes[77..85].copy_from_slice(&u64::MAX.to_le_bytes());
    assert_eq!(
        QueryCorrectionMaterialFrame::readmit(with_checksum(bytes)),
        Err(QueryCorrectionMaterialDenial::MalformedFrame)
    );
}
