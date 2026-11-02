use super::*;
use worth_foundational::{AspectValue, InlineBytes};

#[test]
fn inline_payload_refuses_truncation_trailing_bytes_and_oversized_header() {
    let golden = [24, 3, 0, 0, 0, 0, 128, 255];
    for end in 0..golden.len() {
        assert!(
            decode_aspect_value(&golden[..end]).is_err(),
            "truncation {end}"
        );
    }
    let mut trailing = golden.to_vec();
    trailing.push(1);
    assert!(decode_aspect_value(&trailing).is_err());
    // u32::MAX is an invalid payload width, regardless of available body bytes.
    let error = decode_aspect_value(&[24, 255, 255, 255, 255]).unwrap_err();
    assert!(error.detail().contains("exceeds native framing maximum"));
    let value = decode_aspect_value(&golden).unwrap();
    assert_eq!(
        value,
        AspectValue::InlineBytes(InlineBytes::try_new(vec![0, 128, 255]).unwrap())
    );
    assert_eq!(encoded_aspect_value_len(&value), Some(golden.len()));
}
