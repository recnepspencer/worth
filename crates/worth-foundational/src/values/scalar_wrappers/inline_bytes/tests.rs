use super::*;

#[test]
fn framing_ceiling_accepts_the_boundary_and_refuses_the_next_byte() {
    assert_eq!(InlineBytes::MAXIMUM_LENGTH, 4_294_967_290);
    assert_eq!(
        InlineBytes::admit_length(InlineBytes::MAXIMUM_LENGTH),
        Ok(())
    );
    let denied = InlineBytes::admit_length(InlineBytes::MAXIMUM_LENGTH + 1).unwrap_err();
    assert_eq!(denied.observed(), 4_294_967_291);
    assert_eq!(denied.maximum(), 4_294_967_290);
    // Header admission proves the boundary without allocating a multi-GiB fixture.
    let descriptor =
        serde::de::value::SeqDeserializer::<_, serde::de::value::Error>::new(OversizedHeader);
    assert!(InlineBytes::deserialize(descriptor)
        .unwrap_err()
        .to_string()
        .contains("4294967291"));
}

struct OversizedHeader;
impl Iterator for OversizedHeader {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        panic!("the oversized header must refuse before reading any payload")
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = InlineBytes::MAXIMUM_LENGTH + 1;
        (length, Some(length))
    }
}

#[test]
fn serde_and_construction_preserve_arbitrary_bytes_and_owned_capacity() {
    let mut bytes = Vec::with_capacity(64);
    bytes.extend_from_slice(&[0, 128, 255]);
    let capacity = bytes.capacity();
    let value = InlineBytes::try_new(bytes).unwrap();
    assert_eq!(value.as_bytes(), &[0, 128, 255]);
    assert_eq!(value.wire_length(), 3);
    assert_eq!(value.owned_allocation_capacity_bytes(), capacity);
    assert_eq!(serde_json::to_string(&value).unwrap(), "[0,128,255]");
    assert_eq!(
        serde_json::from_str::<InlineBytes>("[0,128,255]").unwrap(),
        value
    );
}
