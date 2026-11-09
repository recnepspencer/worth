//! Public DATA capture/recovery and independent malformed-record controls.
mod buffered_enums;
mod buffered_values;
mod canonical_refusals;
mod retention_capture;
mod value_corpus;
use crate::facade::application_operation::{
    ApplicationEncodedInput, ApplicationRetainedInputDenial as Denial,
};
use sha2::{Digest, Sha256};
use value_corpus::*;
use worth_foundational::facade::{CanonicalDigestId, ValueCodecDenial, ValueDecodeBounds};
fn bounds() -> ValueDecodeBounds {
    ValueDecodeBounds::new(8192, 1024, 128, 32)
}
fn digest(bytes: &[u8]) -> CanonicalDigestId {
    CanonicalDigestId::new(Sha256::digest(bytes).into())
}
#[test]
fn recorded_input_roundtrips_the_existing_canonical_grammar_with_its_original_identity() {
    let encoded = ApplicationEncodedInput::<CorpusBinding>::encode(corpus()).unwrap();
    assert!(
        encoded.canonical_input().is_none(),
        "ordinary identity policy has no retained capture"
    );
    let capture = encoded.capture_for_retention().unwrap();
    let recorded = capture.recorded_bytes().to_vec();
    let restored = ApplicationEncodedInput::<CorpusBinding>::decode_retained(
        &recorded,
        CanonicalDigestId::new(*encoded.identity()),
        bounds(),
    )
    .unwrap();
    assert_eq!(restored.input(), encoded.input());
    assert_eq!(restored.identity(), encoded.identity());
    assert_eq!(
        restored.capture_for_retention().unwrap().recorded_bytes(),
        capture.recorded_bytes()
    );
}
#[test]
fn retained_input_reuses_the_original_capture_without_requiring_a_second_format() {
    let encoded = ApplicationEncodedInput::<CapturedScalarBinding>::encode(31).unwrap();
    let original = encoded.canonical_input().unwrap();
    let retained = encoded.capture_for_retention().unwrap();
    assert_eq!(retained.recorded_bytes(), original.recorded_bytes());
    let restored = ApplicationEncodedInput::<CapturedScalarBinding>::decode_retained(
        retained.recorded_bytes(),
        CanonicalDigestId::new(*encoded.identity()),
        bounds(),
    )
    .unwrap();
    assert_eq!(*restored.input(), 31);
    assert_eq!(restored.identity(), encoded.identity());
}
#[test]
fn recorded_input_refuses_foreign_binding_protocol_tampering_and_truncation() {
    let encoded = ApplicationEncodedInput::<CorpusBinding>::encode(corpus()).unwrap();
    let captured = encoded.capture_for_retention().unwrap();
    let bytes = captured.recorded_bytes();
    let expected = CanonicalDigestId::new(*encoded.identity());
    assert!(matches!(
        ApplicationEncodedInput::<ForeignBinding>::decode_retained(bytes, expected, bounds()),
        Err(Denial::BindingMismatch)
    ));
    let mut unsupported = bytes.to_vec();
    unsupported[8 + super::super::INPUT_DOMAIN.len() - 1] = b'2';
    assert!(matches!(
        ApplicationEncodedInput::<CorpusBinding>::decode_retained(
            &unsupported,
            digest(&unsupported),
            bounds()
        ),
        Err(Denial::UnsupportedProtocol)
    ));
    let mut altered = bytes.to_vec();
    *altered.last_mut().unwrap() ^= 1;
    assert!(matches!(
        ApplicationEncodedInput::<CorpusBinding>::decode_retained(&altered, expected, bounds()),
        Err(Denial::IdentityMismatch)
    ));
    for end in [0, 7, 8 + super::super::INPUT_DOMAIN.len() - 1] {
        assert!(matches!(
            ApplicationEncodedInput::<CorpusBinding>::decode_retained(
                &bytes[..end],
                expected,
                bounds()
            ),
            Err(Denial::Codec(ValueCodecDenial::Malformed))
        ));
    }
    assert!(matches!(
        ApplicationEncodedInput::<CorpusBinding>::decode_retained(
            &bytes[..bytes.len() - 1],
            expected,
            bounds()
        ),
        Err(Denial::IdentityMismatch)
    ));
    let mut trailing = bytes.to_vec();
    trailing.push(0);
    assert!(matches!(
        ApplicationEncodedInput::<CorpusBinding>::decode_retained(
            &trailing,
            digest(&trailing),
            bounds()
        ),
        Err(Denial::Codec(ValueCodecDenial::NonCanonical))
    ));
}
#[test]
fn recorded_input_decode_applies_its_declared_representation_bounds() {
    let encoded = ApplicationEncodedInput::<CorpusBinding>::encode(corpus()).unwrap();
    let captured = encoded.capture_for_retention().unwrap();
    let bytes = captured.recorded_bytes();
    let expected = CanonicalDigestId::new(*encoded.identity());
    for (policy, refusal) in [
        (
            ValueDecodeBounds::wire_bytes(bytes.len() - 1),
            ValueCodecDenial::WireBytes,
        ),
        (
            ValueDecodeBounds::new(8192, 1, 128, 32),
            ValueCodecDenial::StringBytes,
        ),
        (
            ValueDecodeBounds::new(8192, 1024, 1, 32),
            ValueCodecDenial::SequenceLength,
        ),
        (
            ValueDecodeBounds::new(8192, 1024, 128, 1),
            ValueCodecDenial::Depth,
        ),
    ] {
        assert!(
            matches!(ApplicationEncodedInput::<CorpusBinding>::decode_retained(bytes, expected, policy), Err(Denial::Codec(cause)) if cause == refusal)
        );
    }
}
#[test]
fn recorded_input_revalidates_the_declared_value_after_decoding() {
    let encoded = ApplicationEncodedInput::<PositiveBinding>::encode(1).unwrap();
    let mut bytes = encoded
        .capture_for_retention()
        .unwrap()
        .recorded_bytes()
        .to_vec();
    *bytes.last_mut().unwrap() = 0;
    assert!(
        matches!(ApplicationEncodedInput::<PositiveBinding>::decode_retained(&bytes, digest(&bytes), bounds()), Err(Denial::Encoding(crate::facade::application_schema::ApplicationValueEncodeDenial::Validation(cause))) if cause.reason() == "positive value required")
    );
}
