//! Valid hashes cannot make ambiguous or differently typed bytes canonical.
use super::*;
use std::collections::BTreeMap;
#[test]
fn recorded_input_refuses_overlong_varints_and_a_different_scalar_family() {
    let input = ApplicationEncodedInput::<ScalarBinding>::encode(7).unwrap();
    let original = input.capture_for_retention().unwrap();
    let bytes = original.recorded_bytes();
    let mut overlong = bytes[..bytes.len() - 1].to_vec();
    overlong.extend([0x87, 0]);
    assert!(matches!(
        ApplicationEncodedInput::<ScalarBinding>::decode_retained(
            &overlong,
            digest(&overlong),
            bounds()
        ),
        Err(Denial::Codec(ValueCodecDenial::NonCanonical))
    ));
    let mut signed = bytes.to_vec();
    let tag = signed.len() - 2;
    signed[tag] = super::super::super::encoder::tag::SIGNED;
    assert!(matches!(
        ApplicationEncodedInput::<ScalarBinding>::decode_retained(
            &signed,
            digest(&signed),
            bounds()
        ),
        Err(Denial::Codec(ValueCodecDenial::Malformed))
    ));
}
#[test]
fn recorded_input_refuses_unknown_struct_fields_instead_of_silently_reinterpreting_them() {
    let input = ApplicationEncodedInput::<CorpusBinding>::encode(corpus()).unwrap();
    let original = input.capture_for_retention().unwrap();
    let bytes = original.recorded_bytes();
    let mut extended = bytes[..bytes.len() - 1].to_vec();
    extended.extend([
        1,
        3,
        b'e',
        b'x',
        b't',
        super::super::super::encoder::tag::UNSIGNED,
        7,
        0,
    ]);
    assert!(matches!(
        ApplicationEncodedInput::<CorpusBinding>::decode_retained(
            &extended,
            digest(&extended),
            bounds()
        ),
        Err(Denial::IdentityMismatch)
    ));
}
#[test]
fn recorded_input_refuses_duplicate_map_entries_even_when_the_requested_digest_matches() {
    let input =
        ApplicationEncodedInput::<MapBinding>::encode(BTreeMap::from([("a".into(), 1)])).unwrap();
    let original = input.capture_for_retention().unwrap();
    let bytes = original.recorded_bytes();
    let mut repeated = bytes[..bytes.len() - 1].to_vec();
    repeated.extend([
        1,
        super::super::super::encoder::tag::STR,
        1,
        b'a',
        super::super::super::encoder::tag::UNSIGNED,
        2,
        0,
    ]);
    assert!(matches!(
        ApplicationEncodedInput::<MapBinding>::decode_retained(
            &repeated,
            digest(&repeated),
            bounds()
        ),
        Err(Denial::IdentityMismatch)
    ));
}
