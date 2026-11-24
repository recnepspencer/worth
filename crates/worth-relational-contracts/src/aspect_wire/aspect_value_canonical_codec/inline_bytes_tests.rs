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

#[test]
fn inline_payload_preflight_quotes_owned_bytes_and_refuses_malformed_bodies() {
    let preflight = |bytes: &[u8]| {
        let mut input = bytes.iter().copied();
        value_body_decoded_bytes(bytes.len(), || input.next().ok_or(()), || ())
    };
    let golden = [24, 3, 0, 0, 0, 0, 128, 255];
    let storage = preflight(&golden).unwrap();
    assert_eq!(storage.retained_bytes, 3);
    assert_eq!(storage.transient_bytes, 0);
    assert_eq!(preflight(&[24, 0, 0, 0, 0]).unwrap().retained_bytes, 0);
    for end in 0..golden.len() {
        assert!(preflight(&golden[..end]).is_err(), "truncation {end}");
    }
    let mut trailing = golden.to_vec();
    trailing.push(1);
    assert!(preflight(&trailing).is_err());
}

#[test]
fn inline_payload_preflight_refuses_oversized_length_before_visiting_payload() {
    let length = InlineBytes::MAXIMUM_LENGTH + 1;
    let mut header = vec![24];
    header.extend_from_slice(&(length as u32).to_le_bytes());
    let mut visited = 0;
    // On 32-bit the oversized virtual body itself cannot be represented.
    let body_length = match length.checked_add(5) {
        Some(bytes) => bytes,
        None => usize::MAX,
    };
    assert!(value_body_decoded_bytes(
        body_length,
        || {
            visited += 1;
            header.get(visited - 1).copied().ok_or(())
        },
        || (),
    )
    .is_err());
    assert_eq!(
        visited, 5,
        "the oversized payload is never visited or allocated"
    );
}
