use super::*;

fn prefixed(bytes: &[u8], output: &mut Vec<u8>) {
    output.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    output.extend_from_slice(bytes);
}

fn root() -> Vec<u8> {
    let mut bytes = NAME_DISCRIMINATOR_DOMAIN_TAG.to_vec();
    prefixed(b"id", &mut bytes);
    bytes
}

fn assert_refusal(bytes: &[u8], kind: Kind, offset: usize) {
    assert_eq!(
        scan_name_discriminator_references(bytes),
        Err(DiscriminatorScanError { kind, offset })
    );
}

#[test]
fn reports_absolute_offsets_for_malformed_items() {
    assert_refusal(b"x", Kind::InvalidHeader, 0);
    let start = root().len();
    for (body, kind, delta) in [
        (&[0x10, 2][..], Kind::NonCanonicalBool { found: 2 }, 1),
        (&[0x32, 0x30][..], Kind::MalformedArity, 2),
        (&[0x31, 0x02][..], Kind::MalformedArity, 1),
        (&[0x20, 0][..], Kind::UnexpectedEof, 2),
        (&[0xfe][..], Kind::UnknownTag { found: 0xfe }, 0),
        (&[0x30, 0xff][..], Kind::TrailingBytes { found: 0xff }, 1),
    ] {
        let mut bytes = root();
        bytes.extend_from_slice(body);
        assert_refusal(&bytes, kind, start + delta);
    }
    let mut text = root();
    text.push(0x19);
    prefixed(&[b'a', 0xff], &mut text);
    assert_refusal(&text, Kind::InvalidUtf8, start + 1 + 8 + 1);
}

#[test]
fn enforces_nested_body_boundary_and_reports_offsets_from_the_root() {
    let mut bytes = root();
    bytes.push(0x33);
    prefixed(b"inner", &mut bytes);
    let length_offset = bytes.len();
    assert_refusal(&bytes, Kind::InvalidNestedLength, length_offset);
    prefixed(&[0x10, 2], &mut bytes);
    assert_refusal(
        &bytes,
        Kind::NonCanonicalBool { found: 2 },
        length_offset + 8 + 1,
    );
    assert_refusal(
        &bytes[..bytes.len() - 1],
        Kind::InvalidNestedLength,
        length_offset,
    );
    bytes[length_offset + 7] = 1;
    assert_refusal(&bytes, Kind::UnexpectedEof, length_offset + 8 + 1);
}

#[test]
fn returns_nested_and_outer_references_in_wire_order() {
    let mut inner = vec![0x20];
    inner.extend([1; 32]);
    let mut bytes = root();
    bytes.extend([0x32, 0x33]);
    prefixed(b"inner", &mut bytes);
    prefixed(&inner, &mut bytes);
    bytes.push(0x20);
    bytes.extend([2; 32]);
    assert_eq!(
        scan_name_discriminator_references(&bytes),
        Ok(vec![[1; 32], [2; 32]])
    );
    assert_eq!(scan_name_discriminator_references(&[]), Ok(vec![]));
}
